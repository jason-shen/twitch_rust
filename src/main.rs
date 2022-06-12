use gstreamer as gst;
use gst::prelude::*;

fn main() {
    // initialize gstreamer
    gst::init().unwrap();

    // video source uri
    let uri = "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
    // publish endpoint uri
    let rtmp_uri = "rtmp://live.twitch.tv/app/apikey";

    // create elements for gstreamer
    let source = gst::ElementFactory::make("uridecodebin", Some("source"))
        .expect("Cloud not create sink element uridecodebin");
    let audio = gst::ElementFactory::make("avenc_aac", Some("avenc_aac"))
        .expect("Cloud not create sink element avenc_aac");
    let convert = gst::ElementFactory::make("audioconvert", Some("audioconvert"))
        .expect("Cloud not create sink element audioconvert");
    let resample = gst::ElementFactory::make("audioresample", Some("audioresample"))
        .expect("Cloud not create sink element audioresample");    
    let queuesrc = gst::ElementFactory::make("queue", Some("faac"))
        .expect("Cloud not create sink element queue");
    let videoconvert = gst::ElementFactory::make("videoconvert", Some("videoconvert"))
        .expect("Cloud not create sink element videoconvert");    
    let x264enc = gst::ElementFactory::make("x264enc", Some("x264enc"))
        .expect("Cloud not create sink element x264enc");    
    let flvmux = gst::ElementFactory::make("flvmux", Some("flvmux"))
        .expect("Cloud not create sink element flvmux");
    let queuesink = gst::ElementFactory::make("queue", Some("queue"))
        .expect("Cloud not create sink element queue");      
    let videosink = gst::ElementFactory::make("rtmpsink", Some("rtmpsink"))
        .expect("Cloud not create sink element rtmpsink");   
        
    flvmux.set_property("streamable", true);
    source.set_property("uri", &uri);
    videosink.set_property("location", &rtmp_uri);
    
    // create empty pipeline
    let pipeline = gst::Pipeline::new(Some("twitch-stream"));

    // build the pipeline
    pipeline.add_many(&[&source, &audio, &convert, &resample, &queuesrc, &videoconvert, &x264enc, &flvmux, &queuesink, &videosink])
    .unwrap();
    // gstreamer elements linking for video
    gst::Element::link_many(&[&queuesrc, &videoconvert, &x264enc, &flvmux, &queuesink, &videosink])
        .expect("Video Elements could not be linked");
    // gstream elements linking for audio
    gst::Element::link_many(&[&convert, &resample, &audio, &flvmux])
        .expect("Audio Elements could not be linked");

    // Connect the pad 
    source.connect_pad_added(move |src, src_pad| {
        println!(
            "Recived new pad {} from {}",
            src_pad.name(),
            src.name()
        );
        let new_pad_caps = src_pad
        .current_caps()
        .expect("Failed to get caps of new pad");
        let new_pad_struct = new_pad_caps
        .structure(0)
        .expect("Failed to get first strucutre of caps");
        let new_pad_type = new_pad_struct.name();

        if new_pad_type.starts_with("audio/x-raw") {
            let sink_pad = convert.static_pad("sink")
            .expect("failed to get static sink pad from convert");
        if sink_pad.is_linked() {
            println!("Audio Pad already linked!");
            return;
        }
        let res = src_pad.link(&sink_pad); 
        if res.is_err() {
            println!("type of {} link failed: ", new_pad_type);
        } else {
            println!("Linked successfully type {}: ", new_pad_type);
        }   
        } else if new_pad_type.starts_with("video/x-raw") {
            let sink_pad = queuesrc.static_pad("sink")
            .expect("failed to get static sink pad for queuesrc");
            if sink_pad.is_linked() {
                println!("video pad already linked!");
                return;
            }
            let res = src_pad.link(&sink_pad);
            if res.is_err() {
                println!("type of {} linked failed: ", new_pad_type);
            } else {
                println!("linked successfully type of {}: ", new_pad_type);
            }
        }
    });

    // start playing
    pipeline.set_state(gst::State::Playing)
    .expect("Unable to set the pipeline to the playing state");

    // Wait unit error of EOF
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;
        match msg.view() {
            MessageView::Error(err) => {
                println!(
                    "Error recieved from element {:?} {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                    );
                    break;
            }
            MessageView::StateChanged(state_change) => {
                if state_change
                .src()
                .map(|s| s == pipeline)
                .unwrap_or(false) {
                    println!(
                        "Pipeline state changed from {:?} to {:?}",
                        state_change.old(),
                        state_change.current()
                    )
                }
            }
            MessageView::Eos(_) => break,
            _ => (),
        }
    }
    pipeline.set_state(gst::State::Null)
    .expect("Unable to set the pipeline to the Null state");
}
