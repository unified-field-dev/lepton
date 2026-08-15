use spectra::spectra_schema;

spectra_schema! {
    LeptonAuthFailure {
        store: "lepton",
        table: "lepton_auth_failure",
        version: "0.1.0",
        description: "Auth flow failure sample (ops-id fields only; no PII, passwords, tokens, or free-form messages).",
        level: Warn,
        fields: [
            flow: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            operation: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            error_class: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            provider: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            channel: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
        ],
    }
}
