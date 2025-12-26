use rocket::FromForm;

use super::{GroupKeyDto, SubmitType, TrimmedStr, datetime::BrowserDateTimeDto};

#[derive(FromForm)]
pub struct EditPostDto<'v> {
    #[field(validate = len(3..))]
    pub title_sv: TrimmedStr<'v>,
    #[field(validate = len(3..))]
    pub title_en: TrimmedStr<'v>,
    #[field(validate = len(10..))]
    pub content_sv: TrimmedStr<'v>,
    #[field(validate = len(10..))]
    pub content_en: TrimmedStr<'v>,
    pub publish_time: Option<BrowserDateTimeDto>,
    pub mandate: Option<GroupKeyDto<'v>>,
    pub darkmode_hide: bool,

    pub publish: SubmitType,
}
