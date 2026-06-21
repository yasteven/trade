
// trade/vsta/src/v_ttai/v_account_0_update/view.rs

pub fn view(wat: &dsta::AccountUpdate) 
-> iced::Element<'_, crate::lens::v_dr_r::Message> 
{ let fstr = format!("\nmv_account_0_update: {:#?}", wat);
  iced::widget::Text::new(fstr).into()
}