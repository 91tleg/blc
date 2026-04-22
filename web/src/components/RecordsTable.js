import React from 'react';

const RecordsTable = ({ records }) => {
  return (
    <div className="table-container">
      {records.length === 0 ? (
        <div className="no-records">
          No records yet. Fill out the signup form above to get started!
        </div>
      ) : (
        <table>
          <thead>
            <tr>
              <th>First Name</th>
              <th>Last Name</th>
              <th>Phone</th>
              <th>Email</th>
              <th>Date</th>
            </tr>
          </thead>
          <tbody>
            {records.map((record) => (
              <tr key={record.id}>
                <td>{record.firstName}</td>
                <td>{record.lastName}</td>
                <td>{record.phone}</td>
                <td>{record.email}</td>
                <td>{record.date}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
};

export default RecordsTable;
