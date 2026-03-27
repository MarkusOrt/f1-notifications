date_displays();
event_click_handler();
session_click_handler();
add_event_handler();
custom_selects();
series_filter();
status_filter();
timezone_renders();

function timezone_renders() {
  const tzs = document.querySelectorAll(".curtz");
  for (const tz of tzs) {
    const offset = new Date().getTimezoneOffset();
    const abs = Math.abs(offset);
    const hours = String(Math.floor(abs / 60)).padStart(2, '0');
    const minutes = String(abs % 60).padStart(2, '0');
    const sign = offset <= 0 ? '+' : '-';
    tz.innerText = ` (Offset: ${sign}${hours}:${minutes})`
  }
}

function add_event_handler() {
  const add_event_btn = document.getElementById("add-event");
  const backdrop = document.getElementById("backdrop");
  if (add_event_btn != null) {
    add_event_btn.onclick = () => {
      const add_event_dialog = document.getElementById("add-event-dialog");
      if (add_event_dialog == null || backdrop == null) return;
      const form = add_event_dialog.querySelector("form");
      if (form == null) return;
      form.onsubmit = new_weekend.bind(form);
      const cancel_button = add_event_dialog.querySelector("button[cancel]");
      cancel_button.onclick = (e) => {
        e.preventDefault();
        e.stopPropagation();
        view_transition(() => {
          add_event_dialog.classList.remove("active");
          backdrop.classList.remove("active")
          add_event_dialog.onclick = null;
          backdrop.onclick = null;
        });
      };
      view_transition(() => {
        add_event_dialog.classList.add("active");
        backdrop.classList.add("active");

      })
      backdrop.onclick = () => {
        view_transition(() => {
          add_event_dialog.classList.remove("active");
          backdrop.classList.remove("active")
          add_event_dialog.onclick = null;
          backdrop.onclick = null;
        });
      }
      custom_selects();
    }
  }
}

function add_session_handler() {
  const add_session_btn = document.getElementById("add-session");
  const backdrop = document.getElementById("backdrop");
  if (add_session_btn != null) {
    add_session_btn.onclick = () => {
      const add_session_dialog = document.getElementById("add-session-dialog");
      if (add_session_dialog == null || backdrop == null) return;
      const form = add_session_dialog.querySelector("form");
      if (form == null) return;
      form.onsubmit = (e) => {
        new_session.bind(form)(e, () => {

          add_session_dialog.classList.remove("active");
          backdrop.classList.remove("active")
          add_session_dialog.onclick = null;
          backdrop.onclick = null;
        });
      }
      const cancel_button = add_session_dialog.querySelector("button[cancel]");
      cancel_button.onclick = (e) => {
        e.preventDefault();
        e.stopPropagation();
        view_transition(() => {
          add_session_dialog.classList.remove("active");
          backdrop.classList.remove("active")
          add_session_dialog.onclick = null;
          backdrop.onclick = null;
        });
      };
      view_transition(() => {
        add_session_dialog.classList.add("active");
        backdrop.classList.add("active");

      })
      backdrop.onclick = () => {
        view_transition(() => {
          add_session_dialog.classList.remove("active");
          backdrop.classList.remove("active")
          add_session_dialog.onclick = null;
          backdrop.onclick = null;
        });
      }
      custom_selects();
    }
  }
}

function date_displays() {
  const date_displays = document.querySelectorAll("[data-date]");
  for (const date_display of date_displays) {
    const data = date_display.getAttribute("data-date");
    if (data == null) continue;

    let date_str = new Intl.DateTimeFormat(undefined, {
      day: 'numeric',
      month: 'short',
      year: 'numeric'
    }).format(new Date(data));
    date_display.innerText = date_str;
  }
}

function event_click_handler() {
  const events = document.querySelectorAll(".event[data-id]");

  for (const event of events) {
    event.onclick = null;
    const actions_toggle = event.querySelector(".actions");
    actions_toggle.onclick = null;
    event.onclick = event_click.bind(event);
    actions_toggle.onclick = toggle_actions.bind(actions_toggle);
  }
}

function session_click_handler() {
  const events = document.querySelectorAll(".session[data-id]");

  for (const event of events) {
    const actions_toggle = event.querySelector(".actions");
    actions_toggle.onclick = null;
    actions_toggle.onclick = toggle_actions.bind(actions_toggle);
  }
}

let g_event_id = null;

function toggle_actions(e) {
  e.preventDefault();
  e.stopPropagation();
  const active_toggles = document.querySelectorAll(".actions.active");
  for (const toggle of active_toggles) {
    toggle.classList.remove("active");
  }
  this.classList.add("active");
  document.body.classList.add("non-click");
  const actions = document.querySelector(".actions.active + .actions-display");
  if (actions == null) return;
  actions.onclick = (e) => {
    e.preventDefault();
    e.stopPropagation();
    let data_container = e.target.closest("[data-id]");
    if (data_container == null) return;
    let id = data_container.getAttribute("data-id");
    if (id == null) return;
    const action = e.target.closest("[data-action]");
    if (action != null) {
      let action_data = action.getAttribute("data-action");
      if (action_data == "edit_event") {
        show_edit_dialog(id);
        document.body.classList.remove("non-click");
        this.classList.remove("active");
        actions.onclick = null;
      }
      if (action_data == "delete_event") {
        delete_weekend(id);
        document.body.classList.remove("non-click");
        this.classList.remove("active");
        actions.onclick = null;
      }
      if (action_data == "edit_session") {
        show_edit_session(id);
        document.body.classList.remove("non-click");
        this.classList.remove("active");
        actions.onclick = null;
      }
      if (action_data == "ignore_session") {
        ignore_notify(id, g_event_id);
        document.body.classList.remove("non-click");
        this.classList.remove("active");
        actions.onclick = null;
      }
      if (action_data == "notify_session") {
        notify(id, g_event_id);
        document.body.classList.remove("non-click");
        this.classList.remove("active");
        actions.onclick = null;
      }
      if (action_data == "delete-session") {
        delete_session(id);
        document.body.classList.remove("non-click");
        this.classList.remove("active");
        actions.onclick = null;
      }
    }
  }

  window.onclick = (e) => {
    if (e.target.closest(".actions-display") != null) {
      return;
    }
    actions.onclick = null;
    document.body.classList.remove("non-click");
    this.classList.remove("active");
  }
}

function ignore_notify(id, event_id) {
  fetch(`/sessions/${id}/notif-off`, {
    method: "PUT"
  }).then((res) => {
    if (res.ok) {
      const sessions_container = document.getElementById("sessions-container");
      fetch(`/events/${event_id}/sessions/render`)
        .then((res) => res.text())
        .then((text) => view_transition(() => {
          sessions_container.outerHTML = text;
          post_sessions_loaded();
          session_click_handler();
        }));
    }
  });
}

function notify(id, event_id) {
  fetch(`/sessions/${id}/notif-on`, {
    method: "PUT"
  }).then((res) => {
    if (res.ok) {
      const sessions_container = document.getElementById("sessions-container");
      fetch(`/events/${event_id}/sessions/render`)
        .then((res) => res.text())
        .then((text) => view_transition(() => {
          sessions_container.outerHTML = text;
          post_sessions_loaded();
          session_click_handler();
        }));
    }
  });
}
function event_click() {
  const sessions_container = document.getElementById("sessions-container");
  const actives = document.querySelectorAll(".event.active");
  for (const active of actives) {
    active.classList.remove("active");
  }
  const event_id = this.getAttribute("data-id");
  this.classList.add("active");
  if (event_id == null) return;
  g_event_id = event_id;
  fetch(`/events/${event_id}/sessions/render`)
    .then((res) => res.text())
    .then((text) => view_transition(() => {
      sessions_container.outerHTML = text;
      post_sessions_loaded();
      session_click_handler();
      add_session_handler();
    }));
}

function view_transition(l) {
  if (document.startViewTransition) {
    document.startViewTransition(l);
  } else {
    l();
  }
}

function post_sessions_loaded() {
  const sessions_container = document.getElementById("sessions-container");
  const datetimes = sessions_container.querySelectorAll("[data-time]");
  for (const datetime of datetimes) {
    datetime_handler(datetime);
  }
}

function datetime_handler(datetime) {
  const dt = datetime.getAttribute("data-time");
  if (dt == null) return;
  const new_text = new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(new Date(dt));
  datetime.innerText = new_text;
}

function custom_selects() {
  const selects = document.querySelectorAll(".custom-select");

  for (const select of selects) {
    select.onclick = select_handler.bind(select);
  }
}

function series_filter() {
  const filter_select = document.getElementById("series-filter");
  const status_select = document.getElementById("status-filter");

  if (filter_select == null || status_select == null) return;
  filter_select.valchange = (new_value) => {
    apply_filters(new_value, status_select.getAttribute("value"));
  }
}

function status_filter() {
  const filter_select = document.getElementById("status-filter");
  const series_filter = document.getElementById("series-filter");
  if (filter_select == null || series_filter == null) return;
  filter_select.valchange = (new_value) => {
    apply_filters(series_filter.getAttribute("value"), new_value);
  }
}

function apply_filters(filter_series, filter_status) {
  const events = document.querySelectorAll(".event[data-status][data-series]");
  for (const event of events) {
    if (filter_series == "all" && filter_status == "all") {
      event.classList.remove("hidden");
      continue;
    }

    let series = event.getAttribute("data-series");
    let status = event.getAttribute("data-status");

    if ((series == filter_series && status == filter_status)
      || (filter_series == "all" && status == filter_status)
      || (series == filter_series && filter_status == "all")) {
      event.classList.remove("hidden");
    } else {
      event.classList.add("hidden");
    }

  }
}

function select_handler(e) {
  const active_selects = document.querySelectorAll(".custom-select:has(.options.visible)");
  for (const active of active_selects) {
    let opts = active.querySelector(".options.visible");
    if (opts == null) continue;
    opts.classList.remove("visible");
  }
  const active_value = this.querySelector(".active-value");
  const input = this.querySelector("input");
  const opts = this.querySelector(".options");
  if (opts == null || active_value == null) return;


  document.onclick = (e) => {
    if (e.target == this
      || e.target.closest(".custom-select") != null) {
      return;
    }
    opts.classList.remove("visible");
    document.onclick = null;
    document.body.classList.remove("non-click");
  }
  document.body.classList.add("non-click");
  opts.classList.add("visible");
  if (e.target.hasAttribute("data-value")) {
    this.setAttribute("value", e.target.getAttribute("data-value"));
    input.value = this.getAttribute("value");
    active_value.innerText = e.target.innerText;
    opts.classList.remove("visible");
    document.onclick = null;
    document.body.classList.remove("non-click");
    if (this.valchange != null) {
      this.valchange(this.getAttribute("value"), this);
    }
  }
}

function show_edit_session(id) {
  let edit_dialog = document.querySelector("#edit-dialog");
  if (edit_dialog != null) {
    edit_dialog.remove();
  }

  fetch(`/sessions/${id}/dialog`)
    .then((res) => res.text())
    .then((text) => {
      document.body.innerHTML += text;
      edit_dialog = document.querySelector("#edit-dialog");
      date_displays();
      event_click_handler();
      session_click_handler();
      add_event_handler();
      custom_selects();
      series_filter();
      status_filter();
      timezone_renders();
      datetime_locals();
      const cancel_button = edit_dialog.querySelector("button[cancel]");
      const form = edit_dialog.querySelector("form");
      form.onsubmit = (e) => {
        session_dialog_submit.bind(form)(e, () => {
          edit_dialog.classList.remove("active");
          backdrop.classList.remove("active");
          backdrop.onclick = null;
          cancel_button.onclick = null;
        });
      }
      const backdrop = document.getElementById("backdrop");
      cancel_button.onclick = (e) => {
        e.stopPropagation();
        e.preventDefault();
        view_transition(() => {
          edit_dialog.classList.remove("active");
          backdrop.classList.remove("active");
          backdrop.onclick = null;
          cancel_button.onclick = null;
        });
      }
      custom_selects();
      view_transition(() => {
        edit_dialog.classList.add("active");
        backdrop.classList.add("active");
      });
      backdrop.onclick = () => {
        view_transition(() => {
          edit_dialog.classList.remove("active");
          backdrop.classList.remove("active");
          backdrop.onclick = null;
          cancel_button.onclick = null;
        });
      }
    });
}
function show_edit_dialog(id) {
  let edit_dialog = document.querySelector("#edit-dialog");
  if (edit_dialog != null) {
    edit_dialog.remove();
  }

  fetch(`/events/${id}/dialog`)
    .then((res) => res.text())
    .then((text) => {
      document.body.innerHTML += text;
      edit_dialog = document.querySelector("#edit-dialog");
      date_displays();
      event_click_handler();
      session_click_handler();
      add_event_handler();
      custom_selects();
      series_filter();
      status_filter();
      timezone_renders();
      const cancel_button = edit_dialog.querySelector("button[cancel]");
      const form = edit_dialog.querySelector("form");
      form.onsubmit = dialog_submit.bind(form);
      const backdrop = document.getElementById("backdrop");
      cancel_button.onclick = (e) => {
        e.stopPropagation();
        e.preventDefault();
        view_transition(() => {
          edit_dialog.classList.remove("active");
          backdrop.classList.remove("active");
          backdrop.onclick = null;
          cancel_button.onclick = null;
        });
      }
      custom_selects();
      view_transition(() => {
        edit_dialog.classList.add("active");
        backdrop.classList.add("active");
      });
      backdrop.onclick = () => {
        view_transition(() => {
          edit_dialog.classList.remove("active");
          backdrop.classList.remove("active");
          backdrop.onclick = null;
          cancel_button.onclick = null;
        });
      }
    });
}

function dialog_submit(e) {
  e.preventDefault();
  e.stopPropagation();
  const json = Object.fromEntries(new FormData(this));
  json.start_date = new Date(json.start_date);
  fetch(`/events/${json.event_id}`, {
    method: "PUT",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify(json),
  }).then((res) => {
    if (res.ok) {
      window.location.reload();
    } else {
      (async () => {
        console.error(await res.text());
      })();
    }
  });
}
function session_dialog_submit(e, x) {
  e.preventDefault();
  e.stopPropagation();
  const json = Object.fromEntries(new FormData(this));
  json.start_time = new Date(json.start_time);
  console.log(x);
  fetch(`/sessions/${json.session_id}`, {
    method: "PUT",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify(json),
  }).then((res) => {
    if (res.ok) {
      const sessions_container = document.getElementById("sessions-container");
      fetch(`/events/${g_event_id}/sessions/render`)
        .then((res) => res.text())
        .then((text) => view_transition(() => {
          sessions_container.outerHTML = text;
          post_sessions_loaded();
          session_click_handler();
          x();
        }));
    } else {
      (async () => {
        console.error(await res.text());
      })();
    }
  });
}

function delete_weekend(id) {
  fetch(`/events/${id}`, {
    method: "DELETE"
  }).then((res) => {
    if (res.ok) {
      window.location.reload();
    }
  });
}

function new_weekend(e) {
  e.preventDefault();
  e.stopPropagation();
  const json = Object.fromEntries(new FormData(this));
  json.start_date = new Date(json.start_date);

  fetch("/events/", {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify(json),
  }).then((res) => {
    if (res.ok) {
      window.location.reload();
    } else {

      (async () => {
        console.error(await res.text());
      })();
    }

  });

}

function delete_session(id) {
  fetch(`/sessions/${id}`, {
    method: "DELETE"
  }).then((res) => {
    if (res.ok) {
      const sessions_container = document.getElementById("sessions-container");
      fetch(`/events/${g_event_id}/sessions/render`)
        .then((res) => res.text())
        .then((text) => view_transition(() => {
          sessions_container.outerHTML = text;
          post_sessions_loaded();
          session_click_handler();
          add_session_handler();
        }));
    }
  });
}

function new_session(e, x) {
  e.preventDefault();
  e.stopPropagation();
  const json = Object.fromEntries(new FormData(this));
  json.start_time = new Date(json.start_time);
  json.event_id = +g_event_id;

  fetch("/sessions/", {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify(json),
  }).then((res) => {
    if (res.ok) {
      const sessions_container = document.getElementById("sessions-container");
      fetch(`/events/${g_event_id}/sessions/render`)
        .then((res) => res.text())
        .then((text) => view_transition(() => {
          sessions_container.outerHTML = text;
          post_sessions_loaded();
          session_click_handler();
          add_session_handler();
          x();
        }));
    } else {
      (async () => {
        console.error(await res.text());
      })();
    }

  });

}

function datetime_locals() {
  const tzs = document.querySelectorAll("[data-utc]");

  for (const tz of tzs) {
    const dt = tz.getAttribute("data-utc");
    if (dt == null) continue;
    const date = new Date(dt);
    let month = date.getMonth() + 1;
    if (month < 10) {
      month = `0${month}`;
    }
    let day = date.getDate();
    if (day < 10) {
      day = `0${day}`;
    }
    let hour = date.getHours();
    if (hour < 10) {
      hour = `0${hour}`;
    }
    let minute = date.getMinutes();
    if (minute < 10) {
      minute = `0${minute}`;
    }
    console.log(`${date.getFullYear()}-${month}-${day}T${hour}:${minute}`);
    tz.value = `${date.getFullYear()}-${month}-${day}T${hour}:${minute}`;
  }
}
