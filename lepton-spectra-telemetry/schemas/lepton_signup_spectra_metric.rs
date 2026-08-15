use spectra::spectra_metric;

spectra_metric! {
    LeptonSignup {
        store: "lepton",
        name: "lepton_signup",
        version: "0.1.0",
        description: "Signup completion attempts. Labels: outcome, error_class.",
    }
}
