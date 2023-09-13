use std::str::FromStr;
use yew::prelude::*;
use crate::app::util::{get_value_from_event, get_value_from_input_event};

mod api;
mod util;

#[derive(PartialEq)]
pub enum PaymentMethod {
    Enot,
    Test
}

pub enum PaymentMsg {
    UpdateNick(String),
    UpdateCrd(String),
    UpdatePaymentMethod(String),
    TryPayment
}

pub struct App {
    current_nick: String,
    warn_message: Option<String>,
    crd_amount: u32,
    payment_method: PaymentMethod,
}

impl Component for App {
    type Message = PaymentMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        return Self {
            current_nick: "".to_string(),
            warn_message: None,
            crd_amount: 0,
            payment_method: PaymentMethod::Enot,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PaymentMsg::UpdateNick(v) => self.current_nick = v,
            PaymentMsg::UpdateCrd(v) => {
                if v == "" {
                    self.crd_amount = 0
                } else if let Ok(v) = u32::from_str(&*v){
                    self.crd_amount = v
                }
            }
            PaymentMsg::UpdatePaymentMethod(v) => {
                if v == "enot" {
                    self.payment_method = PaymentMethod::Enot;
                } else if v == "test" {
                    self.payment_method = PaymentMethod::Test;
                }
            }
            PaymentMsg::TryPayment => {
                let mut is_ok = true;

                if self.crd_amount <= 0 {
                    self.warn_message = Some("Неверное количество CRD!".to_string());
                    is_ok = false;
                }

                if self.current_nick == "" {
                    self.warn_message = Some("Введите имя персонажа!".to_string());
                    is_ok = false;
                }

                if is_ok {
                    self.warn_message = None;
                }
            },
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_nick_change = ctx.link().callback(PaymentMsg::UpdateNick);
        let on_nick_input = Callback::from(move |input_event: InputEvent| {
            on_nick_change.emit(get_value_from_input_event(input_event));
        });

        let on_payment_provider_change = ctx.link().callback(PaymentMsg::UpdatePaymentMethod);
        let on_payment_provider_input = Callback::from(move |event: Event| {
            on_payment_provider_change.emit(get_value_from_event(event));
        });

        let on_crd_change = ctx.link().callback(PaymentMsg::UpdateCrd);
        let on_crd_input = Callback::from(move |input_event: InputEvent| {
            on_crd_change.emit(get_value_from_input_event(input_event));
        });

        let r = html!{
            <>
            <div class="sep_b">
            </div>
            <div class="dlg_a">
                <div class="dlg_b">
                    <div class="dlg_hdr">
                    <span class="logo pull-right"></span>
                        <div class="dlg_hdr_txt">
                        <b>{ "Купить CRD"}</b> 
                        </div>
                    </div>
                    <div class="sep_sm"></div>
                    <div class="dlg_r_a">
                        <div class="dlg_r_b">
                            { "Ник:" }
                        </div>
                        <div class="dlg_r_c">
                            <input placeholder="Введите имя персонажа" type="text" id="nick" name="Ник" class="dlg_r_i" oninput={on_nick_input} value={self.current_nick.clone()}/>
                        </div>
                    </div>
                    <div class="sep_sm"></div>
                    <div class="dlg_r_a">
                        <div class="dlg_r_b2">
                            { "CRD:" }
                        </div>
                        <div class="dlg_r_c">
                            <input placeholder="Количество CRD" id="crd" name="CRD" class="dlg_r_i2" oninput={on_crd_input} value={self.crd_amount.to_string()}/>
                        </div>
                    </div>
                    <div class="sep_sm"></div>
                    <div class="dlg_r_a">
                        <div class="dlg_r_b_b">
                            { "Способ оплаты" }
                        </div>
                        <div class="dlg_r_slct">
                            <select name="payments" id="payments" onchange={on_payment_provider_input}>
                                <option value="enot" selected={self.payment_method == PaymentMethod::Enot}>{ "Enot" }</option>
                                <option value="test" selected={self.payment_method == PaymentMethod::Test}>{ "Prime Payments" }</option>
                                <option value="test2" selected={self.payment_method == PaymentMethod::Test}>{ "Hot Skins" }</option>
                            </select>
                        </div>
                    </div>
                    <div class="sep_sm"></div>
                    <div class="sep_sm"></div>
                    <div class="dlg_f">
                        <button class="clear-completed" onclick={ctx.link().callback(|_| PaymentMsg::TryPayment)}>
                            { "Оплатить" }
                        </button>
                    </div>
                    <div class="sep_sm"></div>
                    <div class="dlg_f2">
                        {
                            if let Some(warn) = &self.warn_message {
                                html!{<div class="dlg_f2_t">{warn} </div>}
                            } else {
                                html!{}
                            }
                        }
                    </div>
                </div>
            </div>
            </>
        };

        r
    }
}
