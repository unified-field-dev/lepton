use spectra::spectra_metric;

spectra_metric! {
    LeptonIdentityDelete {
        store: "lepton",
        name: "lepton_identity_delete",
        version: "0.1.0",
        description: "Guarded identity delete. Labels: operation, outcome, error_class.",
    }
}
