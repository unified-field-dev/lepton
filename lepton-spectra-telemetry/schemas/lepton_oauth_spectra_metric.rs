use spectra::spectra_metric;

spectra_metric! {
    LeptonOauth {
        store: "lepton",
        name: "lepton_oauth",
        version: "0.1.0",
        description: "OAuth begin/complete. Labels: provider, intent, stage, outcome, error_class.",
    }
}
