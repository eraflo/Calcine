//! End-to-end: the real gateway on a free port, backed by the mock services.

use std::sync::Arc;
use std::time::Duration;

use calcine_gateway::{
    CALCINE_ORIGINS, Gateway, GatewayOptions, GatewaySettings, KeyScope, KeyStore, NewApiKey,
};
use calcine_mock::MockBackend;
use reqwest::{Client, StatusCode, header};
use serde_json::{Value, json};

struct Harness {
    gateway: Gateway,
    base: String,
    http: Client,
    _dir: TempDir,
}

struct TempDir(std::path::PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn start(settings: GatewaySettings) -> Harness {
    let dir = std::env::temp_dir()
        .join(format!("calcine-gateway-{}", std::process::id()))
        .join(format!("{:?}", std::thread::current().id()).replace(['(', ')'], ""));
    std::fs::create_dir_all(&dir).unwrap();
    let settings_path = dir.join("gateway.json");
    settings.save(&settings_path).unwrap();

    let gateway = Gateway::new(
        MockBackend::services(),
        GatewayOptions {
            keys: Arc::new(KeyStore::in_memory()),
            settings_path: Some(settings_path),
            builtin_origins: CALCINE_ORIGINS.iter().map(|o| (*o).to_owned()).collect(),
        },
    );
    gateway.start().await.unwrap();
    let base = gateway.status().base_url.trim_end_matches("/v1").to_owned();
    Harness {
        gateway,
        base,
        http: Client::new(),
        _dir: TempDir(dir),
    }
}

fn any_port() -> GatewaySettings {
    GatewaySettings {
        port: 0,
        ..GatewaySettings::default()
    }
}

impl Harness {
    fn key(&self, scopes: &[KeyScope], allow_local_files: bool) -> String {
        self.gateway
            .keys()
            .create(NewApiKey {
                name: "Test app".into(),
                scopes: scopes.to_vec(),
                allow_local_files,
            })
            .unwrap()
            .token
    }

    fn chat(&self, token: Option<&str>, body: &Value) -> reqwest::RequestBuilder {
        let request = self
            .http
            .post(format!("{}/v1/chat/completions", self.base))
            .json(body);
        match token {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }
}

fn hello(stream: bool) -> Value {
    json!({ "model": "qualcomm/Qwen3-0.6B", "stream": stream, "messages": [{ "role": "user", "content": "hello" }] })
}

#[tokio::test(flavor = "multi_thread")]
async fn health_is_public_but_models_need_a_key() {
    let h = start(any_port()).await;
    let health = h.http.get(format!("{}/v1/", h.base)).send().await.unwrap();
    assert_eq!(health.status(), StatusCode::OK);

    let anonymous = h
        .http
        .get(format!("{}/v1/models", h.base))
        .send()
        .await
        .unwrap();
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    let error: Value = anonymous.json().await.unwrap();
    assert_eq!(error["error"]["type"], "authentication_error");

    let token = h.key(&[KeyScope::Inference], false);
    let models: Value = h
        .http
        .get(format!("{}/v1/models", h.base))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(models["object"], "list");
    assert!(
        models["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["id"] == "qualcomm/Qwen3-4B:W4A16")
    );

    let wrong = h
        .http
        .get(format!("{}/v1/models", h.base))
        .bearer_auth("calcine_wrong")
        .send()
        .await
        .unwrap();
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread")]
async fn foreign_hosts_and_origins_are_refused() {
    let h = start(any_port()).await;
    let token = h.key(&[KeyScope::Inference], false);

    // DNS rebinding: a page on evil.example resolved to 127.0.0.1.
    let rebinding = h
        .http
        .get(format!("{}/v1/models", h.base))
        .header(header::HOST, "evil.example")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(rebinding.status(), StatusCode::BAD_REQUEST);

    // A web page calling the API directly, even with a stolen key.
    let cross_site = h
        .http
        .get(format!("{}/v1/models", h.base))
        .header(header::ORIGIN, "https://evil.example")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(cross_site.status(), StatusCode::FORBIDDEN);

    // Calcine's own webview passes the preflight.
    let preflight = h
        .http
        .request(
            reqwest::Method::OPTIONS,
            format!("{}/v1/chat/completions", h.base),
        )
        .header(header::ORIGIN, "http://tauri.localhost")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
    assert_eq!(preflight.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        preflight.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "http://tauri.localhost"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn streams_chat_and_logs_usage() {
    let h = start(any_port()).await;
    let token = h.key(&[KeyScope::Inference], false);
    let response = h.chat(Some(&token), &hello(true)).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );
    let body = response.text().await.unwrap();
    assert!(body.contains("reasoning_content") && body.contains("hello"));
    assert!(body.trim_end().ends_with("data:[DONE]"));

    // The log entry is completed once the stream ends.
    let mut entry = None;
    for _ in 0..50 {
        entry = h
            .gateway
            .requests()
            .into_iter()
            .find(|e| e.status.is_some());
        if entry.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let entry = entry.expect("request should be logged");
    assert_eq!(entry.client, "Test app");
    assert_eq!(entry.model.as_deref(), Some("qualcomm/Qwen3-0.6B"));
    assert_eq!(entry.status, Some(200));
    assert!(entry.completion_tokens.is_some_and(|tokens| tokens > 0));
    assert!(entry.first_token_ms.is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn local_files_need_a_trusted_key() {
    let h = start(any_port()).await;
    let body = json!({
        "model": "m",
        "messages": [{ "role": "user", "content": [
            { "type": "image_url", "image_url": { "url": "C:/Users/me/Documents/secret.png" } }
        ] }],
    });

    let restricted = h.key(&[KeyScope::Inference], false);
    let refused = h.chat(Some(&restricted), &body).send().await.unwrap();
    assert_eq!(refused.status(), StatusCode::FORBIDDEN);

    let trusted = h.key(&[KeyScope::Inference], true);
    let accepted = h.chat(Some(&trusted), &body).send().await.unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn without_required_keys_callers_can_only_infer() {
    let h = start(GatewaySettings {
        require_api_key: false,
        ..any_port()
    })
    .await;
    let chat = h.chat(None, &hello(false)).send().await.unwrap();
    assert_eq!(chat.status(), StatusCode::OK);

    let manage = h
        .http
        .get(format!("{}/calcine/v1/models", h.base))
        .send()
        .await
        .unwrap();
    assert_eq!(manage.status(), StatusCode::FORBIDDEN);

    h.gateway.set_require_api_key(true).await.unwrap();
    let locked = h.chat(None, &hello(false)).send().await.unwrap();
    assert_eq!(locked.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread")]
async fn management_needs_the_manage_scope() {
    let h = start(any_port()).await;
    let inference_only = h.key(&[KeyScope::Inference], false);
    let refused = h
        .http
        .get(format!("{}/calcine/v1/jobs", h.base))
        .bearer_auth(&inference_only)
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), StatusCode::FORBIDDEN);

    let manager = h.key(&[KeyScope::Manage], false);
    let pulled = h
        .http
        .post(format!("{}/calcine/v1/models/pull", h.base))
        .bearer_auth(&manager)
        .json(&json!({ "model": "qualcomm/Llama-v3.2-1B-Instruct" }))
        .send()
        .await
        .unwrap();
    assert_eq!(pulled.status(), StatusCode::ACCEPTED);
    let jobs: Value = h
        .http
        .get(format!("{}/calcine/v1/jobs", h.base))
        .bearer_auth(&manager)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(jobs[0]["kind"]["model"], "qualcomm/Llama-v3.2-1B-Instruct");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_taken_port_is_reported() {
    let blocker = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = blocker.local_addr().unwrap().port();
    let dir = std::env::temp_dir().join(format!("calcine-gateway-port-{port}"));
    let path = dir.join("gateway.json");
    GatewaySettings {
        port,
        ..GatewaySettings::default()
    }
    .save(&path)
    .unwrap();
    let gateway = Gateway::new(
        MockBackend::services(),
        GatewayOptions {
            keys: Arc::new(KeyStore::in_memory()),
            settings_path: Some(path),
            builtin_origins: vec![],
        },
    );
    assert!(gateway.start().await.is_err());
    let status = gateway.status();
    assert!(!status.listening);
    assert!(status.error.unwrap().contains(&port.to_string()));
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn moves_to_another_port_and_keeps_the_old_one_when_taken() {
    let harness = start(any_port()).await;
    let free = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

    harness.gateway.set_port(free).await.unwrap();
    let status = harness.gateway.status();
    assert!(status.listening);
    assert_eq!(status.base_url, format!("http://127.0.0.1:{free}/v1"));
    let reply = harness
        .http
        .get(format!("http://127.0.0.1:{free}/v1/"))
        .send()
        .await
        .unwrap();
    assert_eq!(reply.status(), StatusCode::OK);

    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let taken_port = taken.local_addr().unwrap().port();
    let err = harness.gateway.set_port(taken_port).await.unwrap_err();
    assert!(err.contains("already used"), "{err}");
    let status = harness.gateway.status();
    assert!(status.listening);
    assert_eq!(status.base_url, format!("http://127.0.0.1:{free}/v1"));
    assert_eq!(harness.gateway.settings().port, free);

    assert!(harness.gateway.set_port(80).await.is_err());
}

#[tokio::test]
async fn allowed_origins_are_validated_and_take_effect() {
    let harness = start(any_port()).await;
    assert!(
        harness
            .gateway
            .set_allowed_origins(vec!["localhost:3000".into()])
            .is_err()
    );
    harness
        .gateway
        .set_allowed_origins(vec![" HTTP://localhost:3000/ ".into()])
        .unwrap();
    assert_eq!(
        harness.gateway.settings().allowed_origins,
        ["http://localhost:3000"]
    );
    let reply = harness
        .http
        .get(format!("{}/v1/models", harness.base))
        .header(header::ORIGIN, "http://localhost:3000")
        .send()
        .await
        .unwrap();
    // Allowed origin: past the Origin check, stopped by the missing key.
    assert_eq!(reply.status(), StatusCode::UNAUTHORIZED);
}

fn ollama_chat(stream: bool) -> Value {
    json!({
        "model": "qualcomm/Qwen3-0.6B:latest",
        "stream": stream,
        "messages": [{ "role": "user", "content": "hello" }],
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn speaks_ollama_on_the_main_port_with_a_key() {
    let h = start(any_port()).await;
    let version: Value = h
        .http
        .get(format!("{}/api/version", h.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(version["version"].is_string());

    let anonymous = h
        .http
        .get(format!("{}/api/tags", h.base))
        .send()
        .await
        .unwrap();
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);

    let token = h.key(&[KeyScope::Inference], false);
    let tags: Value = h
        .http
        .get(format!("{}/api/tags", h.base))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        tags["models"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["name"] == "qualcomm/Qwen3-4B:W4A16")
    );

    // Streamed by default, as JSON lines ending with statistics.
    let response = h
        .http
        .post(format!("{}/api/chat", h.base))
        .bearer_auth(&token)
        .json(&ollama_chat(true))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/x-ndjson"
    );
    let lines: Vec<Value> = response
        .text()
        .await
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let (last, pieces) = lines.split_last().unwrap();
    assert_eq!(last["done"], true);
    assert!(last["eval_count"].as_u64().is_some_and(|count| count > 0));
    let text: String = pieces
        .iter()
        .filter_map(|piece| piece["message"]["content"].as_str())
        .collect();
    assert!(text.contains("hello"));

    let whole: Value = h
        .http
        .post(format!("{}/api/chat", h.base))
        .bearer_auth(&token)
        .json(&ollama_chat(false))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(whole["done"], true);
    assert!(
        whole["message"]["content"]
            .as_str()
            .unwrap()
            .contains("hello")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn ollama_generate_and_unsupported_routes() {
    let h = start(any_port()).await;
    let token = h.key(&[KeyScope::Inference], false);
    let generate: Value = h
        .http
        .post(format!("{}/api/generate", h.base))
        .bearer_auth(&token)
        .json(&json!({ "model": "qualcomm/Qwen3-0.6B", "prompt": "hello", "stream": false }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(generate["response"].as_str().unwrap().contains("hello"));

    let pull = h
        .http
        .post(format!("{}/api/pull", h.base))
        .bearer_auth(&token)
        .json(&json!({ "model": "x" }))
        .send()
        .await
        .unwrap();
    assert_eq!(pull.status(), StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_ollama_port_needs_no_key_but_only_runs_models() {
    let h = start(GatewaySettings {
        ollama_port_enabled: true,
        ollama_port: 0,
        ..any_port()
    })
    .await;
    let ollama = h
        .gateway
        .status()
        .ollama_url
        .expect("Ollama port listening");

    let root = h.http.get(&ollama).send().await.unwrap();
    assert_eq!(root.text().await.unwrap(), "Ollama is running");

    let tags = h
        .http
        .get(format!("{ollama}/api/tags"))
        .send()
        .await
        .unwrap();
    assert_eq!(tags.status(), StatusCode::OK);

    let chat: Value = h
        .http
        .post(format!("{ollama}/api/chat"))
        .json(&ollama_chat(false))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(chat["done"], true);
    let entry = h.gateway.requests().into_iter().next().unwrap();
    assert_eq!(entry.client, "Ollama app");
    assert_eq!(entry.path, "/api/chat");

    let manage = h
        .http
        .get(format!("{ollama}/calcine/v1/models"))
        .send()
        .await
        .unwrap();
    assert_eq!(manage.status(), StatusCode::FORBIDDEN);

    // The main port still wants a key.
    let main = h
        .http
        .get(format!("{}/api/tags", h.base))
        .send()
        .await
        .unwrap();
    assert_eq!(main.status(), StatusCode::UNAUTHORIZED);

    // Browser pages are refused here too.
    let page = h
        .http
        .get(format!("{ollama}/api/tags"))
        .header(header::ORIGIN, "https://evil.example")
        .send()
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::FORBIDDEN);

    h.gateway.set_ollama_port(false).await.unwrap();
    assert!(h.gateway.status().ollama_url.is_none());
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(
        h.http
            .get(format!("{ollama}/api/tags"))
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .is_err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn enforces_structured_output() {
    let h = start(any_port()).await;
    let token = h.key(&[KeyScope::Inference], false);
    let schema = |required: &str| {
        json!({
            "model": "qualcomm/Qwen3-0.6B",
            "messages": [{ "role": "user", "content": "hello" }],
            "response_format": { "type": "json_schema", "json_schema": { "name": "r", "schema": {
                "type": "object",
                "properties": { "reply": { "type": "string" } },
                "required": [required],
            } } },
        })
    };

    let ok: Value = h
        .chat(Some(&token), &schema("reply"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let content: Value =
        serde_json::from_str(ok["choices"][0]["message"]["content"].as_str().unwrap()).unwrap();
    assert_eq!(content["reply"], "Mock answer");

    let never = h
        .chat(Some(&token), &schema("missing"))
        .send()
        .await
        .unwrap();
    assert_eq!(never.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error: Value = never.json().await.unwrap();
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("$.missing is missing")
    );

    // Streamed clients get the checked JSON as server-sent events.
    let mut streamed = schema("reply");
    streamed["stream"] = json!(true);
    let response = h.chat(Some(&token), &streamed).send().await.unwrap();
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );
    let body = response.text().await.unwrap();
    assert!(body.contains("Mock answer") && body.trim_end().ends_with("data: [DONE]"));
}

#[tokio::test(flavor = "multi_thread")]
async fn ollama_format_is_enforced_too() {
    let h = start(any_port()).await;
    let token = h.key(&[KeyScope::Inference], false);
    let reply: Value = h
        .http
        .post(format!("{}/api/chat", h.base))
        .bearer_auth(&token)
        .json(&json!({
            "model": "qualcomm/Qwen3-0.6B",
            "stream": false,
            "messages": [{ "role": "user", "content": "hello" }],
            "format": { "type": "object", "required": ["reply"] },
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(reply["done"], true);
    let content: Value =
        serde_json::from_str(reply["message"]["content"].as_str().unwrap()).unwrap();
    assert_eq!(content["reply"], "Mock answer");
}

#[tokio::test(flavor = "multi_thread")]
async fn enforces_tool_choice() {
    let h = start(any_port()).await;
    let token = h.key(&[KeyScope::Inference], false);
    let tool = |name: &str, unit: Value| {
        json!({ "type": "function", "function": { "name": name, "parameters": {
            "type": "object",
            "properties": { "city": { "type": "string" }, "unit": unit },
            "required": ["city"],
        } } })
    };
    let weather = tool("get_weather", json!({ "type": "string" }));
    let time = tool("get_time", json!({ "type": "string" }));
    let ask = |tool_choice: Value, tools: Value| {
        json!({
            "model": "qualcomm/Qwen3-0.6B",
            "messages": [{ "role": "user", "content": "Weather in Paris?" }],
            "tools": tools,
            "tool_choice": tool_choice,
        })
    };

    // A named function is the only one offered, and its call is checked.
    let named = ask(
        json!({ "type": "function", "function": { "name": "get_time" } }),
        json!([weather, time]),
    );
    let reply: Value = h
        .chat(Some(&token), &named)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(reply["choices"][0]["finish_reason"], "tool_calls");
    assert_eq!(
        reply["choices"][0]["message"]["tool_calls"][0]["function"]["name"],
        "get_time"
    );

    // "none" takes the tools away: the model answers in text.
    let reply: Value = h
        .chat(Some(&token), &ask(json!("none"), json!([weather])))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(reply["choices"][0]["message"].get("tool_calls").is_none());
    assert!(
        reply["choices"][0]["message"]["content"]
            .as_str()
            .unwrap()
            .contains("mock backend")
    );

    // Arguments that never match the parameters: 422, saying why.
    let strict = tool("get_weather", json!({ "type": "string", "minLength": 10 }));
    let never = h
        .chat(Some(&token), &ask(json!("required"), json!([strict])))
        .send()
        .await
        .unwrap();
    assert_eq!(never.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error: Value = never.json().await.unwrap();
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("get_weather: $.unit")
    );

    // A function that isn't in the tools is the client's mistake.
    let unknown = ask(
        json!({ "type": "function", "function": { "name": "search" } }),
        json!([weather]),
    );
    let response = h.chat(Some(&token), &unknown).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Streamed clients get the call as server-sent events.
    let mut streamed = ask(json!("required"), json!([weather]));
    streamed["stream"] = json!(true);
    let body = h
        .chat(Some(&token), &streamed)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(body.contains("\"tool_calls\"") && body.contains("\"finish_reason\":\"tool_calls\""));
}

#[tokio::test(flavor = "multi_thread")]
async fn forgets_the_oldest_messages_when_asked() {
    let h = start(any_port()).await;
    let token = h.key(&[KeyScope::Inference], false);
    let long = "word ".repeat(1300);
    let conversation = json!([
        { "role": "system", "content": "Be brief." },
        { "role": "user", "content": long },
        { "role": "assistant", "content": "OK" },
        { "role": "user", "content": long },
        { "role": "assistant", "content": "OK" },
        { "role": "user", "content": "hello" },
    ]);
    let ask = |truncation: Option<&str>| {
        let mut body = json!({ "model": "qualcomm/Qwen3-4B", "messages": conversation });
        if let Some(truncation) = truncation {
            body["truncation"] = json!(truncation);
        }
        body
    };

    // Too long for the window: refused, saying how to fix it.
    let refused = h.chat(Some(&token), &ask(None)).send().await.unwrap();
    assert_eq!(refused.status(), StatusCode::BAD_REQUEST);
    let error: Value = refused.json().await.unwrap();
    assert_eq!(error["error"]["code"], "context_length_exceeded");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("truncation")
    );

    // With truncation, the first exchange goes and the reply comes.
    let fitted = h
        .chat(Some(&token), &ask(Some("auto")))
        .send()
        .await
        .unwrap();
    assert_eq!(fitted.status(), StatusCode::OK);
    assert_eq!(fitted.headers()["x-calcine-forgotten-messages"], "2");
    let reply: Value = fitted.json().await.unwrap();
    assert!(reply["choices"][0]["message"]["content"].is_string());

    // A last message that can't fit alone.
    let huge = json!({
        "model": "qualcomm/Qwen3-4B",
        "truncation": "auto",
        "messages": [{ "role": "user", "content": "word ".repeat(3000) }],
    });
    let response = h.chat(Some(&token), &huge).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error: Value = response.json().await.unwrap();
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("doesn't fit")
    );

    // Ollama clients get it without asking, as with Ollama.
    let ollama = h
        .http
        .post(format!("{}/api/chat", h.base))
        .bearer_auth(&token)
        .json(&json!({ "model": "qualcomm/Qwen3-4B", "stream": false, "messages": conversation }))
        .send()
        .await
        .unwrap();
    assert_eq!(ollama.status(), StatusCode::OK);
}
