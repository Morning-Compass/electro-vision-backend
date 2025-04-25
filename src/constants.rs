pub const DOMAIN: &str = "127.0.0.1:3501";
pub const APPLICATION_JSON: &str = "application/json";
pub const CONNECTION_POOL_ERROR: &str = "couldn't get DB connection from pool";
pub const CONFIRMATION_TOKEN_EXIPIRATION_TIME: i64 = 900; // time in seconds
pub const HASH_COST: u8 = 10;
pub const JWT_EXPIRATION_TIME: i64 = 900;
pub const WORKSPACE_INVITATION_EXPIRATION_TIME: i64 = 604800;
pub const PASSWORD_RESET_TOKEN_EXPIRATION_TIME: i64 = 900;
pub const TEST_USERNAME: &str = "tomek";
pub const TEST_EMAIL: &str = "tomek@el-jot.eu";
pub const TEST_PASSWORD: &str = "qazxsw2.";
pub const ROLES: [&str; 2] = ["USER", "ADMIN"];
pub const SMTP: &str = "smtp.gmail.com";
pub const EMAIL_CSS_TEMPLATE: &str = r#"
    body, html {
        margin: 0;
        padding: 0;
        font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
        background-color: #f8fafc;
    }
    .container {
        max-width: 640px;
        margin: 2rem auto;
        background-color: #ffffff;
        border-radius: 12px;
        overflow: hidden;
        box-shadow: 0 4px 24px rgba(0, 0, 0, 0.05);
        border: 1px solid #e2e8f0;
    }
    .header {
        background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
        color: #ffffff;
        padding: 2rem;
        text-align: center;
    }
    .header h1 {
        margin: 0;
        font-weight: 600;
        font-size: 1.75rem;
    }
    .body {
        padding: 2rem;
        color: #334155;
        line-height: 1.6;
    }
    .body p {
        margin: 1rem 0;
        font-size: 1rem;
    }
    .action-button {
        display: inline-block;
        margin: 1.5rem 0;
        padding: 0.75rem 1.5rem;
        background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
        color: #ffffff !important;
        text-decoration: none;
        border-radius: 8px;
        font-weight: 600;
        transition: all 0.2s ease;
        box-shadow: 0 2px 4px rgba(30, 64, 175, 0.2);
    }
    .action-button:hover {
        transform: translateY(-1px);
        box-shadow: 0 4px 8px rgba(30, 64, 175, 0.25);
    }
    .footer {
        padding: 1.5rem;
        text-align: center;
        font-size: 0.875rem;
        color: #64748b;
        background-color: #f8fafc;
        border-top: 1px solid #e2e8f0;
    }
    .footer p {
        margin: 0.5rem 0;
    }
    .footer a {
        color: #2563eb;
        text-decoration: none;
        font-weight: 500;
    }
    .text-muted {
        color: #94a3b8;
        font-size: 0.875rem;
    }
    @media only screen and (max-width: 600px) {
        .container {
            margin: 0;
            border-radius: 0;
            border: none;
        }
        .header, .body, .footer {
            padding: 1.5rem;
        }
    }
"#;
