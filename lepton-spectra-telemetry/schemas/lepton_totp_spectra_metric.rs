use spectra::spectra_metric;

spectra_metric! {
    LeptonTotp {
        store: "lepton",
        name: "lepton_totp",
        version: "0.1.0",
        description: "TOTP enroll/disable/verify. Labels: operation, outcome, error_class.",
    }
}
