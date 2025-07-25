use std::{collections::HashMap, env, time::Duration};

use fantoccini::{Client, ClientBuilder, Locator, key::Key};
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{self, ChatCompletionRequest},
    common::O3,
    types,
};
use scraper::{Html, Selector};
use tokio::time::{Instant, sleep};
use url::Url;

use crate::domain::website::{
    models::website::{Contact, GeneratedWebsite, WebsiteAiError},
    ports::WebsiteAi,
};

#[derive(Clone)]
pub struct Ai {
    webdriver_address: String,
    lovable_user: String,
    lovable_password: String,
}

impl Ai {
    pub fn new(webdriver_addres: &str, lovable_user: &str, lovable_password: &str) -> Self {
        Self {
            webdriver_address: webdriver_addres.to_string(),
            lovable_user: lovable_user.to_string(),
            lovable_password: lovable_password.to_string(),
        }
    }

    pub async fn wait_until_lovable_preview_disappears(
        &self,
        webdriver: &Client,
        timeout: Duration,
    ) -> Result<(), fantoccini::error::CmdError> {
        let xpath = "//span[normalize-space(.)='Spinning up preview...']";
        let start = Instant::now();

        while start.elapsed() < timeout {
            match webdriver.find(Locator::XPath(xpath)).await {
                Ok(_) => {
                    sleep(Duration::from_millis(250)).await;
                }
                Err(fantoccini::error::CmdError::WaitTimeout) => {
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }

        Err(fantoccini::error::CmdError::WaitTimeout)
    }
}

impl From<fantoccini::error::CmdError> for WebsiteAiError {
    fn from(value: fantoccini::error::CmdError) -> Self {
        WebsiteAiError::WebdriverError(value)
    }
}

impl WebsiteAi for Ai {
    async fn get_full_website(
        &self,
        website_source_address: &str,
    ) -> Result<String, WebsiteAiError> {
        tracing::debug!("getting full website for {}", website_source_address);
        let mut visited: HashMap<Url, String> = HashMap::new();
        let url =
            Url::parse(website_source_address).map_err(|_| WebsiteAiError::FailedToFetchContent)?;
        let website_source_code = reqwest::get(url.to_owned())
            .await
            .map_err(|_| WebsiteAiError::FailedToFetchContent)?
            .text()
            .await
            .map_err(|_| WebsiteAiError::FailedToFetchContent)?;
        visited.insert(url.clone(), website_source_code.clone());

        let (to_visit, external_links) = {
            let selector = Selector::parse("a[href]").unwrap();
            let fragment = Html::parse_fragment(&website_source_code);

            let mut internals = Vec::new();
            let mut externals = Vec::new();

            for el in fragment.select(&selector) {
                if let Some(href) = el.value().attr("href") {
                    if href.starts_with("http") {
                        if let Ok(u) = Url::parse(href) {
                            externals.push(u);
                        }
                        continue;
                    }

                    if href.starts_with('/') && href != "/" {
                        if let Ok(u) = Url::parse(&format!("{website_source_address}{href}")) {
                            internals.push(u);
                        }
                    }
                }
            }

            (internals, externals)
        };

        for url in to_visit.iter() {
            let website_source_code = reqwest::get(url.to_owned())
                .await
                .map_err(|_| WebsiteAiError::FailedToFetchContent)?
                .text()
                .await
                .map_err(|_| WebsiteAiError::FailedToFetchContent)?;
            visited.insert(url.clone(), website_source_code.clone());
        }

        let mut full_content = String::new();
        for page in visited.iter() {
            full_content.push_str("==== ");
            full_content.push_str(page.0.as_ref());
            full_content.push_str(" ==== \n");
            full_content.push_str(page.1);
            full_content.push_str("\n\n");
        }
        full_content.push_str("\n\n==== Related links ==== \n");
        for external_link in external_links.iter() {
            full_content.push_str(external_link.as_ref());
            full_content.push('\n');
        }
        Ok(full_content)
    }
    async fn get_contact(&self, full_website: &str) -> Result<Contact, WebsiteAiError> {
        tracing::debug!("getting contact for {}", full_website);
        let api_key = env::var("OPENAI_API_KEY")
            .expect("OpenAI key not found. Should be saved in an env var called OPENAI_API_KEY");
        let mut client = OpenAIClient::builder()
            .with_api_key(api_key)
            .build()
            .map_err(|_| WebsiteAiError::FailedToInitOpeanAi)?;

        let mut properties = HashMap::new();
        properties.insert(
            "contact_name".to_owned(),
            Box::new(types::JSONSchemaDefine {
                schema_type: Some(types::JSONSchemaType::String),
                description: Some(
                    "First name of the website owner, if absent, your best guess on who the company owner might be, if abset the company name ".to_string(),
                ),
                ..Default::default()
            }),
        );
        properties.insert(
            "contact_email".to_owned(),
            Box::new(types::JSONSchemaDefine {
                schema_type: Some(types::JSONSchemaType::String),
                description: Some("Email address of the owner or main contact".to_string()),
                ..Default::default()
            }),
        );
        properties.insert(
            "social_links".to_owned(),
            Box::new(types::JSONSchemaDefine {
                schema_type: Some(types::JSONSchemaType::Object),
                description: Some(
                    "Social and review links on the site, only if present in the sent html"
                        .to_string(),
                ),
                properties: Some(HashMap::from([
                    (
                        "instagram".to_owned(),
                        Box::new(types::JSONSchemaDefine {
                            schema_type: Some(types::JSONSchemaType::String),
                            ..Default::default()
                        }),
                    ),
                    (
                        "facebook".to_owned(),
                        Box::new(types::JSONSchemaDefine {
                            schema_type: Some(types::JSONSchemaType::String),
                            ..Default::default()
                        }),
                    ),
                    (
                        "google_review".to_owned(),
                        Box::new(types::JSONSchemaDefine {
                            schema_type: Some(types::JSONSchemaType::String),
                            ..Default::default()
                        }),
                    ),
                    (
                        "google_maps".to_owned(),
                        Box::new(types::JSONSchemaDefine {
                            schema_type: Some(types::JSONSchemaType::String),
                            ..Default::default()
                        }),
                    ),
                ])),
                ..Default::default()
            }),
        );

        let req = ChatCompletionRequest::new(
            O3.to_string(),
            vec![chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(format!(
                    "Extract the owner’s contact details from the following HTML. \
         If you find a personal name use it, otherwise use the company \
         name. Return the data **only** via the function.\n\n{full_website}\n"
                )),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
        )
        .tools(vec![chat_completion::Tool {
            r#type: chat_completion::ToolType::Function,
            function: types::Function {
                name: String::from("save_site_contacts"),
                description: Some(String::from(
                    "Stores contact information found on a website",
                )),
                parameters: types::FunctionParameters {
                    schema_type: types::JSONSchemaType::Object,
                    properties: Some(properties),
                    required: Some(vec![String::from("website_source_code")]),
                },
            },
        }])
        .tool_choice(chat_completion::ToolChoiceType::Auto);

        let result = client
            .chat_completion(req)
            .await
            .map_err(|e| WebsiteAiError::Unknown(e.into()))?;
        match result.choices[0].finish_reason {
            Some(chat_completion::FinishReason::tool_calls) => {
                if let Some(tool_call) = result.choices[0].message.tool_calls.iter().next() {
                    let arguments = tool_call[0].function.arguments.clone().unwrap();
                    dbg!(&arguments);
                    let contact: Contact = serde_json::from_str(&arguments)
                        .map_err(|e| WebsiteAiError::Unknown(e.into()))?;
                    dbg!(&contact);
                    return Ok(contact);
                }
            }
            _ => {
                tracing::debug!("error 1");
                return Err(WebsiteAiError::FailedToFetchContact);
            }
        }
        tracing::debug!("not matching any");
        Err(WebsiteAiError::FailedToFetchContact)
    }

    async fn generate_new_single_page(
        &self,
        website_source_address: &str,
    ) -> Result<GeneratedWebsite, WebsiteAiError> {
        tracing::debug!("generating new single page");
        let webdriver = ClientBuilder::native()
            .connect(&self.webdriver_address)
            .await
            .expect("failed to connect to WebDriver");

        webdriver.goto("https://lovable.dev/login").await?;

        webdriver
            .find(Locator::Id("email"))
            .await?
            .send_keys(&self.lovable_user)
            .await?;

        webdriver
            .find(Locator::Id("password"))
            .await?
            .send_keys(&self.lovable_password)
            .await?;
        webdriver
            .find(Locator::XPath("//button[normalize-space()='Log in']"))
            .await?
            .click()
            .await?;

        webdriver
            .wait()
            .at_most(Duration::from_secs(30))
            .for_element(Locator::Id("chatinput"))
            .await?;

        let prompt = format!(
            r#"
You are a senior conversion‑focused web designer + copywriter. Starting from the website {website_source_address}, produce one modern, responsive, accessible landing page.
Research: audience, core offer, pains, differentiators, social proof—invent plausible placeholders if missing.
Brand: derive clean style; fix weak colors for accessible palette; modern typography, white space, subtle animation.
Structure (omit if irrelevant): Hero (benefit headline + primary CTA) → Trust logos → Problem → Solution/Benefits (bullets) → Social Proof → Pricing/Offer → FAQ (4–6) → Secondary CTA + contact form → Footer.
Copy: concise, persuasive, second‑person, outcome‑headed; ≥3 CTA placements.
CTAs: high‑contrast (≥7:1) solid primary + outlined secondary; clear hover.
Tech: mobile‑first; optimized images/placeholders; meta title/description; form (name/email/message) with validation. 
"#
        )
        .replace("\n", " ");

        let chat_form = webdriver
            .wait()
            .at_most(Duration::from_secs(30))
            .for_element(Locator::XPath("(//textarea)[1]"))
            .await?;
        for ch in prompt.chars() {
            chat_form.send_keys(&ch.to_string()).await?;
            sleep(Duration::from_millis(1)).await;
        }
        chat_form.send_keys(&prompt).await?;
        chat_form.send_keys(&format!("{}", Key::Enter)).await?;

        sleep(Duration::from_secs(10)).await;

        let timeout = Duration::from_secs(600);
        self.wait_until_lovable_preview_disappears(&webdriver, timeout)
            .await?;
        let name = webdriver
            .find(fantoccini::Locator::XPath("//*[@id='main-menu']//p[1]"))
            .await?
            .text()
            .await?;
        Ok(GeneratedWebsite {
            name,
            url: webdriver.current_url().await?,
        })
    }
}
