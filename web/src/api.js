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

export const registerForEvent = async ({ firstName, lastName, email, phone }) => {
  if (!isBackendConfigured) {
    return null;
  }

  const fullName = [firstName, lastName].filter(Boolean).join(' ').trim();
  const body = {
    full_name: fullName,
    email
  };

  if (phone) {
    body.phone_number = phone;
  }

  const response = await fetch(
    `${apiBaseUrl}/events/${encodeURIComponent(eventId)}/registrations`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(body)
    }
  );

  const data = await readJson(response);

  if (!response.ok) {
    throw new Error(getErrorMessage(response, data));
  }

  return data;
};
