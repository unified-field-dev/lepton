use spectra::spectra_metric;

spectra_metric! {
    LeptonEmailSend {
        store: "lepton",
        name: "lepton_email_send",
        version: "0.1.0",
        description: "Email delivery attempts that reached a terminal outcome. Labels: driver, outcome.",
    }
}
