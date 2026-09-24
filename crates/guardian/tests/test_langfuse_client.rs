use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_langfuse_client_from_env_missing_keys_returns_none() {
    let _guard = ENV_LOCK.lock().unwrap();
    let old_pk = std::env::var("LANGFUSE_PUBLIC_KEY").ok();
    let old_sk = std::env::var("LANGFUSE_SECRET_KEY").ok();

    std::env::remove_var("LANGFUSE_PUBLIC_KEY");
    std::env::remove_var("LANGFUSE_SECRET_KEY");

    let result = guardian::integration::langfuse::LangfuseClient::from_env();

    if let Some(v) = old_pk {
        std::env::set_var("LANGFUSE_PUBLIC_KEY", v);
    }
    if let Some(v) = old_sk {
        std::env::set_var("LANGFUSE_SECRET_KEY", v);
    }

    assert!(result.is_none());
}

#[test]
fn test_langfuse_client_from_env_valid_keys_returns_instance() {
    let _guard = ENV_LOCK.lock().unwrap();
    let old_pk = std::env::var("LANGFUSE_PUBLIC_KEY").ok();
    let old_sk = std::env::var("LANGFUSE_SECRET_KEY").ok();

    std::env::set_var("LANGFUSE_PUBLIC_KEY", "pk-lf-test");
    std::env::set_var("LANGFUSE_SECRET_KEY", "sk-lf-test");

    let result = guardian::integration::langfuse::LangfuseClient::from_env();

    if let Some(v) = old_pk {
        std::env::set_var("LANGFUSE_PUBLIC_KEY", v);
    } else {
        std::env::remove_var("LANGFUSE_PUBLIC_KEY");
    }
    if let Some(v) = old_sk {
        std::env::set_var("LANGFUSE_SECRET_KEY", v);
    } else {
        std::env::remove_var("LANGFUSE_SECRET_KEY");
    }

    assert!(result.is_some());
}
