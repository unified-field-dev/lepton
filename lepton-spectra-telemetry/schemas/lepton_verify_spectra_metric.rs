use spectra::spectra_metric;

spectra_metric! {
    LeptonVerify {
        store: "lepton",
        name: "lepton_verify",
        version: "0.1.0",
        description: "Email/phone/TOTP verification. Labels: channel, stage, outcome, error_class.",
    }
}
