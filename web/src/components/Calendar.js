import React from 'react';
import ReactCalendar from 'react-calendar';
import 'react-calendar/dist/Calendar.css';

const Calendar = ({ selectedDate, onDateChange }) => {
  return (
    <div className="calendar-wrapper">
      <ReactCalendar
        value={selectedDate}
        onChange={onDateChange}
        locale="en-US"
        calendarType="gregory"
        showNeighboringMonth={true}
      />
    </div>
  );
};

export default Calendar;
