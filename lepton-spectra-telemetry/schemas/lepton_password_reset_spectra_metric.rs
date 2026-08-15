use spectra::spectra_metric;

spectra_metric! {
    LeptonPasswordReset {
        store: "lepton",
        name: "lepton_password_reset",
        version: "0.1.0",
        description: "Password reset request/confirm. Labels: stage, outcome, error_class.",
    }
}
