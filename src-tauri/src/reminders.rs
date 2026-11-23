//! Напоминания на Android: ежедневные уведомления в заданное время.
//! На компьютере и на сайте их нет (сайт не умеет будить телефон без push-сервера).

use minus27_core::state::Reminder;
use tauri::{AppHandle, Runtime};

/// id уведомлений: 100 + номер напоминания в списке. Старые снимаются перед установкой новых.
#[cfg(mobile)]
const BASE_ID: i32 = 100;

#[cfg(mobile)]
pub fn apply<R: Runtime>(app: &AppHandle<R>, list: &[Reminder]) -> Result<bool, String> {
    use tauri_plugin_notification::{NotificationExt, PermissionState, Schedule, ScheduleInterval};

    let n = app.notification();
    let ids: Vec<i32> = (0..10).map(|i| BASE_ID + i).collect();
    n.cancel(ids).map_err(|e| e.to_string())?;
    if !list.iter().any(|r| r.on) {
        return Ok(true);
    }
    let granted = match n.permission_state().map_err(|e| e.to_string())? {
        PermissionState::Granted => true,
        _ => n.request_permission().map_err(|e| e.to_string())? == PermissionState::Granted,
    };
    if !granted {
        return Ok(false);
    }
    for (i, r) in list.iter().enumerate().filter(|(_, r)| r.on) {
        n.builder()
            .id(BASE_ID + i as i32)
            .title(r.kind.title())
            .body(r.kind.body())
            .schedule(Schedule::Interval {
                interval: ScheduleInterval { hour: Some(r.hour), minute: Some(r.minute), second: Some(0), ..Default::default() },
                allow_while_idle: true,
            })
            .show()
            .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

/// На компьютере напоминаний нет: просто сообщаем UI, что ничего не включено.
#[cfg(not(mobile))]
pub fn apply<R: Runtime>(_app: &AppHandle<R>, _list: &[Reminder]) -> Result<bool, String> {
    Ok(false)
}
