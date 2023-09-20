use std::str::FromStr;
use gloo_console::log;
use yew::prelude::*;
use shared::{InvoiceCreationResponse, PaymentServices};
use crate::app::api::BackendApi;
use crate::app::util::{get_value_from_event, get_value_from_input_event};

mod api;
mod util;

const MIN_CRD: u32 = 20;

pub enum PaymentMsg {
    UpdateNick(String),
    UpdateCrd(String),
    UpdatePaymentMethod(String),
    TryPayment,
    LinkOk(String),
    LinkErr(String),
}

pub struct App {
    current_nick: String,
    warn_message: Option<String>,
    crd_amount: u32,
    payment_method: PaymentServices,
}

impl Component for App {
    type Message = PaymentMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        return Self {
            current_nick: "".to_string(),
            warn_message: None,
            crd_amount: MIN_CRD,
            payment_method: PaymentServices::Enot,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PaymentMsg::UpdateNick(v) => self.current_nick = v,
            PaymentMsg::UpdateCrd(v) => {
                if v == "" {
                    self.crd_amount = MIN_CRD
                } else if let Ok(v) = u32::from_str(&*v){
                    self.crd_amount = v
                }
            }
            PaymentMsg::UpdatePaymentMethod(v) => {
                if v == "enot" {
                    self.payment_method = PaymentServices::Enot;
                }
            }
            PaymentMsg::TryPayment => {
                let mut is_ok = true;

                if self.crd_amount < MIN_CRD {
                    self.warn_message = Some(format!("Минимум {MIN_CRD} CRD!"));
                    is_ok = false;
                }

                if self.current_nick == "" {
                    self.warn_message = Some("Введите имя персонажа!".to_string());
                    is_ok = false;
                }

                if is_ok {
                    self.warn_message = None;

                    let name = self.current_nick.clone();
                    let amount = self.crd_amount;
                    let method = self.payment_method;

                    ctx.link().send_future(async move {
                        match BackendApi::create_invoice(name, amount, method).await {
                            Ok(resp) => {
                                match resp {
                                    InvoiceCreationResponse::Ok(v) => {
                                        PaymentMsg::LinkOk(v)
                                    }
                                    InvoiceCreationResponse::WrongNick => {
                                        PaymentMsg::LinkErr("Неверное имя персонажа!".to_string())
                                    }
                                    InvoiceCreationResponse::Err => {
                                        PaymentMsg::LinkErr("Network error".to_string())
                                    }
                                }
                            }
                            Err(e) => {
                                log!(format!("{e:#?}"));
                                PaymentMsg::LinkErr("Network error".to_string())
                            }
                        }
                    });
                }
            },
            PaymentMsg::LinkOk(url) => {
                web_sys::window().unwrap().location().replace(&url).unwrap();
            }
            PaymentMsg::LinkErr(err) => {
                self.warn_message = Some(err)
            }
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
                        <div class= "dragon"></div>
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
                                <option value="enot" selected={self.payment_method == PaymentServices::Enot}>{ "Enot" }</option>
                            </select>
                        </div>
                    </div>
                    <div class="sep_sm"></div>
                    <div class="sep_sm"></div>
                    <div class="dlg_f">
                        <button class="fill" onclick={ctx.link().callback(|_| PaymentMsg::TryPayment)}>
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
