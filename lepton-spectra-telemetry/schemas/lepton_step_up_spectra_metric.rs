use spectra::spectra_metric;

spectra_metric! {
    LeptonStepUp {
        store: "lepton",
        name: "lepton_step_up",
        version: "0.1.0",
        description: "Step-up factor verify. Labels: path, outcome, error_class.",
    }
}
