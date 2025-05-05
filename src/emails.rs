use crate::constants::EMAIL_CSS_TEMPLATE;

pub enum EmailType {
    AccountVerification(String, String),
    AccountVerificationResend(String, String),
    ChangePassword(String, String),
    ChangePasswordResend(String, String),
    WorkspaceInvitation(String, String),
}

pub fn email_body_generator(email_type: EmailType) -> String {
    match email_type {
        EmailType::AccountVerification(username, token) => format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>{}</style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>Welcome to Electro-Vision</h1>
        </header>
        <section class="body">
            <article>
                <p>Hello {}!</p>
                <p>Thank you for creating an account with <strong>Electro-Vision</strong>! To start exploring our products and services, please verify your email address by clicking the button below:</p>
                <p style="text-align: center;">
                    <a href="{}" class="action-button">Verify Your Email</a>
                </p>
                <p class="text-muted">This link is valid for 15 minutes</p>
                <p>If you didn't create this account, you can safely ignore this email.</p>
                <p>Welcome aboard! We’re excited to have you with us.</p>
                <p>If you can't click button copy and paste this link</p>
                <p>{}</p>
                <p>Best Regards,<br>Electro-Vision Team</p>
            </article>
        </section>
        <footer class="footer">
            <p>Need help? <a href="mailto:support@electro-vision.com">Contact Support</a></p>
            <p>Electro-Vision, Szczecin</p>
        </footer>
    </div>
</body>
</html>
            "#,
            EMAIL_CSS_TEMPLATE, username, token, token
        ),

        EmailType::AccountVerificationResend(username, token) => format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>{}</style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>Confirm Your Email - Electro-Vision</h1>
        </header>
        <section class="body">
            <article>
                <p>Hello {}!</p>
                <p>We noticed you haven't verified your email address yet. To complete registration, please click below:</p>
                <p style="text-align: center;">
                    <a href="{}" class="action-button">Verify Your Email</a>
                </p>
                <p class="text-muted">This link is valid for 15 minutes.</p>
                <p>If you didn’t request this email, please ignore it.</p>
                <p>If you can't click button copy and paste this link</p>
                <p>{}</p>
                <p>Best Regards,<br>Electro-Vision Team</p>
            </article>
        </section>
        <footer class="footer">
            <p>Need help? <a href="mailto:support@electro-vision.com">Contact Support</a></p>
            <p>Electro-Vision, Szczecin</p>
        </footer>
    </div>
</body>
</html>
            "#,
            EMAIL_CSS_TEMPLATE, username, token, token
        ),

        EmailType::ChangePassword(username, token) => format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>{}</style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>Electro-Vision Account Security</h1>
        </header>
        <section class="body">
            <article>
                <p>Hello {},</p>
                <p>We received a password change request. Click below to proceed:</p>
                <p style="text-align: center;">
                    <a href="{}" class="action-button">Change password</a>
                </p>
                <p class="text-muted">Link expires in 15 minutes. Ignore if not requested.</p>

                <p>If you can't click button copy and paste this link</p>
                <p>{}</p>
                <p>Need help? <a href="mailto:support@electro-vision.com">Contact support</a></p>
            </article>
        </section>
        <footer class="footer">
            <p>© 2023 Electro-Vision · Szczecin</p>
            <p>Ensuring your digital vision</p>
        </footer>
    </div>
</body>
</html>
            "#,
            EMAIL_CSS_TEMPLATE, username, token, token
        ),

        EmailType::ChangePasswordResend(username, token) => format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>{}</style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>Password Change Reminder</h1>
        </header>
        <section class="body">
            <article>
                <p>Hello {},</p>
                <p>Here's your new password change link as requested:</p>
                <p style="text-align: center;">
                    <a href="{}" class="action-button">Change password</a>
                </p>
                <p class="text-muted">This link will expire in 15 minutes.</p>
                <p>If you didn't request this, please secure your account.</p>

                <p>If you can't click button copy and paste this link</p>
                <p>{}</p>
                <p>Best Regards,<br>Electro-Vision Team</p>
            </article>
        </section>
        <footer class="footer">
            <p>Need assistance? <a href="mailto:support@electro-vision.com">Contact us</a></p>
            <p>Electro-Vision, Szczecin</p>
        </footer>
    </div>
</body>
</html>
            "#,
            EMAIL_CSS_TEMPLATE, username, token, token
        ),
        EmailType::WorkspaceInvitation(username, token) => format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>{}</style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>Workspace Invitation</h1>
        </header>
        <section class="body">
            <article>
                <p>Hello {},</p>
                <p>You have been invited to join a workspace</p>
                <p style="text-align: center;">
                    <a href="{}" class="action-button">Accept</a>
                </p>
                <p class="text-muted">This link will expire in 7 days.</p>
                <p>Best Regards,<br>Electro-Vision Team</p>
            </article>
        </section>
        <footer class="footer">
            <p>Need assistance? <a href="mailto:support@electro-vision.com">Contact us</a></p>
            <p>Electro-Vision, Szczecin</p>
        </footer>
    </div>
</body>
</html>
            "#,
            EMAIL_CSS_TEMPLATE, username, token
        ),
    }
}
