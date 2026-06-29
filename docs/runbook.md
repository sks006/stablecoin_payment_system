# Runbook

## Emergency Mitigations

- **Pause Minting**: Call the governance script to trigger the global `paused` flag in the PDA.
- **Key Compromise**: Rotate the AWS KMS signer key via Terraform and redeploy the orchestrator.
