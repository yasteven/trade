
// trade/vsta/src/v_ttai/v_order_update_vec/view.rs

pub fn view(wat: &Vec<dsta::PlacedOrder>)
-> iced::Element<'_, crate::lens::v_dr_r::Message> 
{ let fstr = format!("\nmv_order_update_vec: {:#?}", wat);
  iced::widget::Text::new(fstr).into()
}