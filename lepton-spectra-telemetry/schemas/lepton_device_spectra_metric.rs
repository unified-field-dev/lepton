use spectra::spectra_metric;

spectra_metric! {
    LeptonDevice {
        store: "lepton",
        name: "lepton_device",
        version: "0.1.0",
        description: "TrustedBrowser/WebAuthn device ops. Labels: device_kind, operation, outcome, error_class.",
    }
}
