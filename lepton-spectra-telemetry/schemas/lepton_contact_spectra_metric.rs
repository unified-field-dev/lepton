use spectra::spectra_metric;

spectra_metric! {
    LeptonContact {
        store: "lepton",
        name: "lepton_contact",
        version: "0.1.0",
        description: "Contact add/primary/verify/delete. Labels: channel, operation, outcome, error_class.",
    }
}
