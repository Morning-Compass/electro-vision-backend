use crate::constants::EMAIL_CSS_TEMPLATE;

pub enum EmailType {
    AccountVerification(String, String),
    AccountVerificationResend(String, String),
    ChangePassword,
    ChangePasswordResend,
}

pub fn email_body_generator(email_type: EmailType) -> String {
    match email_type {
        EmailType::AccountVerification(username, token) => {
            return format!(
                r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
    {}
    </style>
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
                <a href="{}" class="verify-button">Verify Your Email</a>
            </p>
            <p>This link is valid for 15 minutes</p>
            <p>If you didn't create this account, you can safely ignore this email.</p>
            <p>Welcome aboard! We’re excited to have you with us.</p>
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
                EMAIL_CSS_TEMPLATE, username, token
            );
        }
        EmailType::AccountVerificationResend(username, token) => {
            return format!(
                r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
    {}
    </style>
</head>
<body>

<div class="container">
    <header class="header">
        <h1>Confirm Your Email - Electro-Vision</h1>
    </header>

    <section class="body">
        <article>
            <p>Hello {}!</p>
            <p>We noticed that you haven't verified your email address for your Electro-Vision account yet. To complete your registration, please verify your email by clicking the button below:</p>
            <p style="text-align: center;">
                <a href="{}" class="verify-button">Verify Your Email</a>
            </p>
            <p>This link is valid for 15 minutes.</p>
            <p>If you didn’t request this email, you can safely ignore it.</p>
            <p>We look forward to having you onboard.</p>
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
                EMAIL_CSS_TEMPLATE, username, token
            );
        }
        EmailType::ChangePassword => todo!(),
        EmailType::ChangePasswordResend => todo!(),
    }
}
