use spectra::spectra_metric;

spectra_metric! {
    LeptonSignin {
        store: "lepton",
        name: "lepton_signin",
        version: "0.1.0",
        description: "Sign-in funnel stages. Labels: stage, outcome, error_class, factor.",
    }
}
