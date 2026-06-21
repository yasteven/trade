
// trade/vsta/src/v_ttai/v_all_my_positions/view.rs

pub fn view(wat: &dsta::PositionsList) 
-> iced::Element<'_, crate::lens::v_dr_r::Message> 
{ let fstr = format!("\nmv_all_my_positions: {:#?}", wat);
  iced::widget::Text::new(fstr).into()
}