use anyhow::{Context, Result};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};
use tracing::{debug, info};
use warp::Filter;

/// OAuth callback result
#[derive(Debug)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

/// OAuth callback server for handling Google OAuth redirects
pub struct CallbackServer {
    port: u16,
    callback_tx: Arc<RwLock<Option<oneshot::Sender<OAuthCallback>>>>,
}

impl CallbackServer {
    /// Create new callback server
    ///
    /// # Arguments
    /// * `port` - Port to listen on (typically 9090)
    pub fn new(port: u16) -> Self {
        Self {
            port,
            callback_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the callback server and wait for OAuth callback
    ///
    /// Returns the authorization code and state when received
    pub async fn start_and_wait(&self) -> Result<OAuthCallback> {
        info!("Starting OAuth callback server on port {}", self.port);

        // Create oneshot channel for callback
        let (tx, rx) = oneshot::channel();
        {
            let mut callback_tx = self.callback_tx.write().await;
            *callback_tx = Some(tx);
        }

        // Clone Arc for the route handler
        let callback_tx_clone = Arc::clone(&self.callback_tx);

        // Define callback route
        let callback_route = warp::path("oauth")
            .and(warp::path("callback"))
            .and(warp::query::<CallbackParams>())
            .map(move |params: CallbackParams| {
                let callback_tx = Arc::clone(&callback_tx_clone);

                tokio::spawn(async move {
                    let mut tx_lock = callback_tx.write().await;
                    if let Some(tx) = tx_lock.take() {
                        let _ = tx.send(OAuthCallback {
                            code: params.code,
                            state: params.state,
                        });
                    }
                });

                warp::reply::html(SUCCESS_HTML)
            });

        // Bind to address
        let addr = (Ipv4Addr::new(127, 0, 0, 1), self.port);

        debug!("Callback server listening on http://localhost:{}", self.port);

        // Start server in background
        let server = warp::serve(callback_route);
        let (_, server_task) = server.bind_with_graceful_shutdown(addr, async {
            // Server will shutdown when we receive callback
        });

        tokio::spawn(server_task);

        // Wait for callback
        let callback = rx
            .await
            .context("Failed to receive OAuth callback")?;

        info!("OAuth callback received successfully");

        // Give server time to send response
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        Ok(callback)
    }

    /// Start server in background (non-blocking)
    ///
    /// Use `wait_for_callback()` to wait for the result
    pub async fn start(&self) -> Result<()> {
        info!("Starting OAuth callback server on port {}", self.port);

        let callback_tx_clone = Arc::clone(&self.callback_tx);

        let callback_route = warp::path("oauth")
            .and(warp::path("callback"))
            .and(warp::query::<CallbackParams>())
            .map(move |params: CallbackParams| {
                let callback_tx = Arc::clone(&callback_tx_clone);

                tokio::spawn(async move {
                    let mut tx_lock = callback_tx.write().await;
                    if let Some(tx) = tx_lock.take() {
                        let _ = tx.send(OAuthCallback {
                            code: params.code,
                            state: params.state,
                        });
                    }
                });

                warp::reply::html(SUCCESS_HTML)
            });

        let addr = (Ipv4Addr::new(127, 0, 0, 1), self.port);

        // Start server in background
        tokio::spawn(async move {
            warp::serve(callback_route).run(addr).await;
        });

        debug!("Callback server started on http://localhost:{}", self.port);

        Ok(())
    }

    /// Wait for OAuth callback
    ///
    /// Must be called after `start()`
    pub async fn wait_for_callback(&self) -> Result<OAuthCallback> {
        let (tx, rx) = oneshot::channel();
        {
            let mut callback_tx = self.callback_tx.write().await;
            *callback_tx = Some(tx);
        }

        let callback = rx
            .await
            .context("Failed to receive OAuth callback")?;

        info!("OAuth callback received");

        Ok(callback)
    }
}

/// Query parameters from OAuth callback
#[derive(Debug, serde::Deserialize)]
struct CallbackParams {
    code: String,
    state: String,
}

/// HTML response shown to user after successful OAuth
const SUCCESS_HTML: &str = r##"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>LoLShorts - Authorization Successful</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }
        .container {
            background: white;
            border-radius: 16px;
            padding: 48px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
            text-align: center;
            max-width: 500px;
        }
        .success-icon {
            width: 80px;
            height: 80px;
            margin: 0 auto 24px;
            background: #10b981;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .checkmark {
            width: 40px;
            height: 40px;
            border: 4px solid white;
            border-left: none;
            border-top: none;
            transform: rotate(45deg);
            margin-top: -10px;
        }
        h1 {
            color: #1f2937;
            margin: 0 0 16px;
            font-size: 28px;
        }
        p {
            color: #6b7280;
            margin: 0 0 32px;
            font-size: 16px;
            line-height: 1.6;
        }
        .button {
            background: #667eea;
            color: white;
            padding: 12px 32px;
            border-radius: 8px;
            text-decoration: none;
            display: inline-block;
            font-weight: 600;
            transition: background 0.2s;
        }
        .button:hover {
            background: #5568d3;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="success-icon">
            <div class="checkmark"></div>
        </div>
        <h1>Authorization Successful!</h1>
        <p>Your YouTube account has been successfully connected to LoLShorts. You can now close this window and return to the application.</p>
        <button class="button" onclick="window.close(); return false;">Close Window</button>
    </div>
    <script>
        setTimeout(function() { window.close(); }, 3000);
    </script>
</body>
</html>"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_callback_server_creation() {
        let server = CallbackServer::new(9090);
        assert_eq!(server.port, 9090);
    }

    // Note: Full integration tests require actual HTTP requests
    // These would be better suited for E2E tests
}
