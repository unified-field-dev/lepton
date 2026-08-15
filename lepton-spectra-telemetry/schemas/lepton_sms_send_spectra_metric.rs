use spectra::spectra_metric;

spectra_metric! {
    LeptonSmsSend {
        store: "lepton",
        name: "lepton_sms_send",
        version: "0.1.0",
        description: "SMS delivery attempts that reached a terminal outcome. Labels: driver, outcome.",
    }
}
