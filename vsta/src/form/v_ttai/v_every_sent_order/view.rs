
// trade/vsta/src/v_ttai/v_every_sent_order/view.rs

pub fn view(wat: &Vec<(dsta::Order, dsta::PushOrderResult)>) 
-> iced::Element<'_, crate::lens::v_dr_r::Message> 
{ let fstr = format!("\nmv_every_sent_order: {:#?}", wat);
  iced::widget::Text::new(fstr).into()
}