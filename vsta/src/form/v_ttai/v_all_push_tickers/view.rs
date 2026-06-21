
// trade/vsta/src/v_ttai/v_all_push_tickers/view.rs

pub fn view(wat: &std::collections::HashMap<String, dsta::PushTickerResult>)
-> iced::Element<'_, crate::lens::v_dr_r::Message> 
{ let fstr = format!("\nmv_all_push_tickers: {:#?}", wat);
  iced::widget::Text::new(fstr).into()
}