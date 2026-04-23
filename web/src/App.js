import React, { useState, useEffect } from 'react';
import './App.css';
import SignupForm from './components/SignupForm';
import RecordsTable from './components/RecordsTable';
import Calendar from './components/Calendar';
import EventPosters from './components/EventPosters';
import * as XLSX from 'xlsx';
import {
  deleteEventPoster,
  getConfiguredEvent,
  isBackendConfigured,
  listEventPosters,
  listRegistrations,
  registerForEvent,
  uploadEventPoster
} from './api';

function App() {
  const [records, setRecords] = useState(() => {
    const saved = localStorage.getItem('blcRecords');
    return saved ? JSON.parse(saved) : [];
  });

  const [eventPostersByDate, setEventPostersByDate] = useState(() => {
    try {
      const saved = localStorage.getItem('blcEventPosters');
      return saved ? JSON.parse(saved) : {};
    } catch {
      return {};
    }
  });

  const [eventNamesByDate, setEventNamesByDate] = useState(() => {
    try {
      const saved = localStorage.getItem('blcEventNames');
      return saved ? JSON.parse(saved) : {};
    } catch {
      return {};
    }
  });

  const [posterStorageError, setPosterStorageError] = useState('');
  const [rotating, setRotating] = useState(false);
  const [selectedDate, setSelectedDate] = useState(() => getLocalMidnight(new Date()));
  const [showCalendar, setShowCalendar] = useState(false);

  function getLocalMidnight(value) {
    const date = new Date(value);
    return new Date(date.getFullYear(), date.getMonth(), date.getDate());
  }

  function getLocalDateKey(value) {
    const date = getLocalMidnight(value);
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }

  function formatDisplayDate(date) {
    const options = { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' };
    return getLocalMidnight(date).toLocaleDateString('en-US', options);
  }

  const getSelectedDateString = () => {
    return getLocalDateKey(selectedDate);
  };

  useEffect(() => {
    localStorage.setItem('blcRecords', JSON.stringify(records));
  }, [records]);

  useEffect(() => {
    try {
      localStorage.setItem('blcEventPosters', JSON.stringify(eventPostersByDate));
      setPosterStorageError('');
    } catch {
      setPosterStorageError('Poster storage is full. Remove a poster or upload a smaller image.');
    }
  }, [eventPostersByDate]);

  useEffect(() => {
    localStorage.setItem('blcEventNames', JSON.stringify(eventNamesByDate));
  }, [eventNamesByDate]);

  useEffect(() => {
    if (!isBackendConfigured) {
      return;
    }

    let cancelled = false;

    const loadBackendState = async () => {
      try {
        const [event, backendRegistrations, backendPosters] = await Promise.all([
          getConfiguredEvent(),
          listRegistrations(),
          listEventPosters()
        ]);

        if (cancelled) {
          return;
        }

        setRecords(backendRegistrations.map(registration => (
          registrationToRecord(registration, event?.starts_at)
        )));
        setEventPostersByDate(groupPostersByDate(backendPosters));

        if (event?.name) {
          const eventDateKey = event.starts_at
            ? getLocalDateKey(event.starts_at)
            : getSelectedDateString();
          setEventNamesByDate(prev => ({
            ...prev,
            [eventDateKey]: prev[eventDateKey] || event.name
          }));
        }

        setPosterStorageError('');
      } catch (error) {
        setPosterStorageError(error.message || 'Unable to load backend event data.');
      }
    };

    loadBackendState();

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const splitFullName = (fullName) => {
    const parts = (fullName || '').trim().split(/\s+/).filter(Boolean);
    if (parts.length === 0) {
      return { firstName: 'N/A', lastName: 'N/A' };
    }

    return {
      firstName: parts[0],
      lastName: parts.slice(1).join(' ') || 'N/A'
    };
  };

  const registrationToRecord = (registration, eventDate) => {
    const submittedAt = registration.registered_at || new Date().toISOString();
    const recordDate = getLocalMidnight(eventDate || submittedAt);
    const names = splitFullName(registration.full_name);

    return {
      id: registration.registration_id,
      firstName: names.firstName,
      lastName: names.lastName,
      phone: registration.phone_number || 'N/A',
      email: registration.email || 'N/A',
      date: formatDisplayDate(recordDate),
      dateKey: getLocalDateKey(recordDate),
      timestamp: recordDate.toISOString(),
      submittedAt
    };
  };

  const backendPosterToView = (poster) => {
    const uploadedAt = poster.uploaded_at || new Date().toISOString();
    return {
      id: poster.poster_id || poster.id,
      name: poster.name,
      url: poster.url,
      dateKey: poster.date_key,
      uploadedAt
    };
  };

  const groupPostersByDate = (posters) => {
    return (posters || []).reduce((grouped, poster) => {
      const viewPoster = backendPosterToView(poster);
      const dateKey = viewPoster.dateKey || getLocalDateKey(viewPoster.uploadedAt);
      return {
        ...grouped,
        [dateKey]: [...(grouped[dateKey] || []), viewPoster]
      };
    }, {});
  };

  const getRecordsByDate = (date) => {
    const dateKey = getLocalDateKey(date);
    return records.filter(record => {
      if (record.dateKey) {
        return record.dateKey === dateKey;
      }

      return getLocalDateKey(record.timestamp) === dateKey;
    });
  };

  const getEventPostersByDate = (date) => {
    return eventPostersByDate[getLocalDateKey(date)] || [];
  };

  const getEventNameByDate = (date) => {
    return eventNamesByDate[getLocalDateKey(date)] || '';
  };

  const handleSave = async (formData) => {
    const recordDate = getLocalMidnight(selectedDate);
    const backendRegistration = await registerForEvent(formData);

    setRotating(true);
    setTimeout(() => setRotating(false), 800);

    const newRecord = {
      id: backendRegistration?.registration_id || Date.now(),
      firstName: formData.firstName || 'N/A',
      lastName: formData.lastName || 'N/A',
      phone: backendRegistration?.phone_number || formData.phone || 'N/A',
      email: backendRegistration?.email || formData.email || 'N/A',
      date: formatDisplayDate(recordDate),
      dateKey: getLocalDateKey(recordDate),
      timestamp: recordDate.toISOString(),
      submittedAt: backendRegistration?.registered_at || new Date().toISOString()
    };

    setRecords(prevRecords => [newRecord, ...prevRecords]);
    return newRecord;
  };

  const handleAddPosters = async (posters) => {
    const dateKey = getSelectedDateString();
    const savedPosters = isBackendConfigured
      ? await Promise.all(posters.map(poster => uploadEventPoster({ ...poster, dateKey })))
      : posters;
    const viewPosters = savedPosters.map(poster => (
      poster?.poster_id ? backendPosterToView(poster) : poster
    ));

    setEventPostersByDate(prev => ({
      ...prev,
      [dateKey]: [...(prev[dateKey] || []), ...viewPosters]
    }));
  };

  const handleEventNameChange = (event) => {
    const dateKey = getSelectedDateString();
    const value = event.target.value;

    setEventNamesByDate(prev => {
      if (!value.trim()) {
        const nextNames = { ...prev };
        delete nextNames[dateKey];
        return nextNames;
      }

      return {
        ...prev,
        [dateKey]: value
      };
    });
  };

  const handleReplacePoster = async (posterId, poster) => {
    const dateKey = getSelectedDateString();
    if (isBackendConfigured) {
      await deleteEventPoster(posterId);
    }

    const savedPoster = isBackendConfigured
      ? backendPosterToView(await uploadEventPoster({ ...poster, dateKey }))
      : { ...poster, id: posterId, uploadedAt: new Date().toISOString() };

    setEventPostersByDate(prev => ({
      ...prev,
      [dateKey]: (prev[dateKey] || []).map(existingPoster => (
        existingPoster.id === posterId
          ? savedPoster
          : existingPoster
      ))
    }));
  };

  const handleRemovePoster = async (posterId) => {
    const dateKey = getSelectedDateString();
    if (isBackendConfigured) {
      await deleteEventPoster(posterId);
    }

    setEventPostersByDate(prev => {
      const nextPosters = (prev[dateKey] || []).filter(poster => poster.id !== posterId);
      if (nextPosters.length > 0) {
        return { ...prev, [dateKey]: nextPosters };
      }

      const remainingDates = { ...prev };
      delete remainingDates[dateKey];
      return remainingDates;
    });
  };

  const handleDateChange = (date) => {
    setSelectedDate(getLocalMidnight(date));
    setShowCalendar(false);
  };

  const formatSubmittedAt = (value) => {
    if (!value) {
      return 'N/A';
    }

    return new Date(value).toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit'
    });
  };

  const handleExport = () => {
    const selectedRecords = getRecordsByDate(selectedDate);
    const selectedPosters = getEventPostersByDate(selectedDate);
    const selectedEventName = getEventNameByDate(selectedDate);

    if (selectedRecords.length === 0 && selectedPosters.length === 0 && !selectedEventName) {
      alert('No event or signup records to export for the selected date');
      return;
    }

    const signupRows = [
      ['BLC Event Signup Export'],
      ['Event Name', selectedEventName || 'Untitled event'],
      ['Event Date', formatDisplayDate(selectedDate)],
      ['Event Posters', selectedPosters.length > 0 ? selectedPosters.map(poster => poster.name).join(', ') : 'No event poster uploaded'],
      ['Total Signups', selectedRecords.length],
      [],
      ['#', 'First Name', 'Last Name', 'Phone', 'Email', 'Submitted At'],
      ...selectedRecords.map((record, index) => [
        index + 1,
        record.firstName,
        record.lastName,
        record.phone,
        record.email,
        formatSubmittedAt(record.submittedAt)
      ])
    ];

    const eventRows = [
      ['BLC Event Details'],
      ['Event Name', selectedEventName || 'Untitled event'],
      ['Event Date', formatDisplayDate(selectedDate)],
      ['Poster Count', selectedPosters.length],
      [],
      ['#', 'Poster File', 'Uploaded At'],
      ...selectedPosters.map((poster, index) => [
        index + 1,
        poster.name,
        formatSubmittedAt(poster.uploadedAt)
      ])
    ];

    const wb = XLSX.utils.book_new();
    const signupSheet = XLSX.utils.aoa_to_sheet(signupRows);
    signupSheet['!cols'] = [
      { wch: 6 },
      { wch: 20 },
      { wch: 20 },
      { wch: 20 },
      { wch: 34 },
      { wch: 24 }
    ];
    signupSheet['!merges'] = [{ s: { r: 0, c: 0 }, e: { r: 0, c: 5 } }];
    signupSheet['!autofilter'] = {
      ref: `A7:F${Math.max(signupRows.length, 7)}`
    };

    const eventSheet = XLSX.utils.aoa_to_sheet(eventRows);
    eventSheet['!cols'] = [
      { wch: 6 },
      { wch: 45 },
      { wch: 24 }
    ];
    eventSheet['!merges'] = [{ s: { r: 0, c: 0 }, e: { r: 0, c: 2 } }];
    eventSheet['!autofilter'] = {
      ref: `A6:C${Math.max(eventRows.length, 6)}`
    };

    XLSX.utils.book_append_sheet(wb, signupSheet, 'Signups');
    XLSX.utils.book_append_sheet(wb, eventSheet, 'Event Posters');
    XLSX.writeFile(wb, `blc_event_${getSelectedDateString()}.xlsx`);
  };

  const selectedRecords = getRecordsByDate(selectedDate);
  const selectedPosters = getEventPostersByDate(selectedDate);
  const selectedDateLabel = formatDisplayDate(selectedDate);
  const selectedEventName = getEventNameByDate(selectedDate);

  return (
    <div className="app">
      <div className="event-name-bar">
        <label className="event-title-group">
          <span className="event-title-label">Event Name</span>
          <input
            type="text"
            value={selectedEventName}
            onChange={handleEventNameChange}
            placeholder="Enter event name"
            aria-label="Event name"
            className="event-name-input"
          />
        </label>
        <span className="event-date-bar">{selectedDateLabel}</span>
      </div>

      <div className="event-entry-layout">
        <SignupForm
          onSave={handleSave}
          isRotating={rotating}
          selectedDateLabel={selectedDateLabel}
          backendEnabled={isBackendConfigured}
        />
        <aside className="event-poster-panel">
          <EventPosters
            selectedDateLabel={selectedDateLabel}
            posters={selectedPosters}
            onAddPosters={handleAddPosters}
            onReplacePoster={handleReplacePoster}
            onRemovePoster={handleRemovePoster}
            storageError={posterStorageError}
          />
        </aside>
      </div>

      <div className="overview-section">
        <h2>Event Records</h2>
        <div className="filter-controls">
          <div className="date-display">
            {selectedDateLabel}
          </div>
          <button onClick={() => setShowCalendar(!showCalendar)} className="btn-calendar">Pick Date</button>
          {showCalendar && <Calendar selectedDate={selectedDate} onDateChange={handleDateChange} />}
          <button onClick={handleExport} className="btn-export">Export to Excel</button>
        </div>
        <RecordsTable records={selectedRecords} />
      </div>
    </div>
  );
}

export default App;
