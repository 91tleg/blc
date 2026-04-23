const apiBaseUrl = (process.env.REACT_APP_API_BASE_URL || '').replace(/\/+$/, '');
const eventId = process.env.REACT_APP_EVENT_ID || '';

export const isBackendConfigured = Boolean(apiBaseUrl && eventId);

const readJson = async (response) => {
  const text = await response.text();
  if (!text) {
    return null;
  }

  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
};

const getErrorMessage = (response, data) => {
  return data?.error?.message || `Request failed with status ${response.status}`;
};

const apiUrl = (path) => {
  return `${apiBaseUrl}${path}`;
};

const requestJson = async (path, options = {}) => {
  const response = await fetch(apiUrl(path), options);
  const data = await readJson(response);

  if (!response.ok) {
    throw new Error(getErrorMessage(response, data));
  }

  return data;
};

export const getConfiguredEvent = async () => {
  if (!isBackendConfigured) {
    return null;
  }

  return requestJson(`/events/${encodeURIComponent(eventId)}`);
};

export const listRegistrations = async () => {
  if (!isBackendConfigured) {
    return [];
  }

  const registrations = [];
  let cursor = '';

  do {
    const query = new URLSearchParams({ limit: '200' });
    if (cursor) {
      query.set('cursor', cursor);
    }

    const data = await requestJson(
      `/events/${encodeURIComponent(eventId)}/registrations?${query.toString()}`
    );
    registrations.push(...(data?.registrations || []));
    cursor = data?.next_cursor || '';
  } while (cursor);

  return registrations;
};

export const listEventPosters = async () => {
  if (!isBackendConfigured) {
    return [];
  }

  const data = await requestJson(`/events/${encodeURIComponent(eventId)}/posters`);
  return data?.posters || [];
};

export const uploadEventPoster = async ({ name, dataUrl, dateKey }) => {
  if (!isBackendConfigured) {
    return null;
  }

  return requestJson(`/events/${encodeURIComponent(eventId)}/posters`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      name,
      data_url: dataUrl,
      date_key: dateKey
    })
  });
};

export const deleteEventPoster = async (posterId) => {
  if (!isBackendConfigured || !posterId) {
    return null;
  }

  return requestJson(
    `/events/${encodeURIComponent(eventId)}/posters/${encodeURIComponent(posterId)}`,
    {
      method: 'DELETE'
    }
  );
};

export const registerForEvent = async ({ firstName, lastName, email, phone, dateKey }) => {
  if (!isBackendConfigured) {
    return null;
  }

  const fullName = [firstName, lastName].filter(Boolean).join(' ').trim();
  const body = {
    full_name: fullName,
    email
  };

  const normalizedPhone = (phone || '').replace(/\D/g, '');
  if (normalizedPhone) {
    body.phone_number = normalizedPhone;
  }
  if (dateKey) {
    body.date_key = dateKey;
  }

  return requestJson(`/events/${encodeURIComponent(eventId)}/registrations`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(body)
  });
};
