use spectra::spectra_metric;

spectra_metric! {
    LeptonAccount {
        store: "lepton",
        name: "lepton_account",
        version: "0.1.0",
        description: "Account lifecycle ops. Labels: operation, outcome, error_class.",
    }
}
