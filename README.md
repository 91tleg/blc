# BLC

This repository contains the Bellevue College Business Leadership Community app:

- Rust AWS Lambda backend at the repository root
- React signup/dashboard frontend in `web/`

The frontend works locally with browser storage by default. If `REACT_APP_API_BASE_URL` and `REACT_APP_EVENT_ID` are configured, signups are also submitted to the backend registration endpoint.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)
- Node.js 22+ and npm

## Frontend

```bash
cd web
npm install
npm start
```

Create `web/.env` from `web/.env.example` to connect the frontend to a deployed backend:

```bash
REACT_APP_API_BASE_URL=https://your-api-id.execute-api.your-region.amazonaws.com/your-stage
REACT_APP_EVENT_ID=evt_your_event_id
```

Build the frontend:

```bash
cd web
npm run build
```

## Backend

To build the Lambda for production, run `cargo lambda build --release`. Remove the `--release` flag to build for development.

Read more about building your lambda function in [the Cargo Lambda documentation](https://www.cargo-lambda.info/commands/build.html).

## Testing

You can run regular Rust unit tests with `cargo test`.

For the frontend:

```bash
cd web
npm test -- --watchAll=false --passWithNoTests
```

## Deploying

The deploy workflows target `us-east-1`.

Required GitHub repository secrets:

- `AWS_DEPLOY_ROLE_ARN`: IAM role ARN for GitHub Actions OIDC. This must be a role ARN, not the OIDC provider ARN.
- `WEB_API_URL`: API Gateway invoke URL after the backend deploy creates `blc-api-prod`.
- `WEB_EVENT_ID`: Event ID to send public frontend signups to, once an event exists.

Configured frontend deploy targets:

- S3 bucket: `blc-private-site-isra-2026`
- CloudFront distribution: `E3JTKIJUG03POH`

Required AWS SSM SecureString parameters:

- `/blc/prod/admin-password`
- `/blc/prod/jwt-secret`

To deploy the backend manually, run `cargo lambda deploy`. This will create an IAM role and a Lambda function in your AWS account.

Read more about deploying your lambda function in [the Cargo Lambda documentation](https://www.cargo-lambda.info/commands/deploy.html).
