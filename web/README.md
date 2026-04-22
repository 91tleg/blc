# BLC Dashboard - Business Leadership Community

A modern, interactive React web dashboard for the Business Leadership Community at Bellevue College.

## Features

✨ **Signup Form**
- First Name and Last Name fields (at least one required)
- Phone number with country code selector (default +1 for US)
- Email input with customizable domain (default @bellevuecollege.edu)
- Interactive, form validation with real-time feedback
- Save and Clear buttons

📊 **Records Management**
- Display all submitted signups in a sortable table
- Shows First Name, Last Name, Phone, Email, and Date
- Shows "N/A" for unavailable fields

📅 **Date Filtering**
- Filter records by date range using calendar inputs
- Clear filters to view all records

📥 **Export Function**
- Export filtered records to CSV
- Download directly to your computer

🔌 **Optional Backend**
- Rust AWS Lambda backend lives at the repository root
- Frontend keeps working locally with LocalStorage if no backend env vars are set
- Set `REACT_APP_API_BASE_URL` and `REACT_APP_EVENT_ID` to submit signups to the backend

## Installation & Setup

### Prerequisites
- Node.js (v14 or higher)
- npm (comes with Node.js)

### Installation Steps

1. Navigate to the frontend directory:
```bash
cd "c:\Users\isras\OneDrive\Desktop\Bellevue College\Spring 2026\BLC\web"
```

2. Install dependencies:
```bash
npm install
```

3. Start the development server:
```bash
npm start
```

4. Open your browser and navigate to:
```
http://localhost:3000
```

### Backend Connection

The app works like the original local version by default. To connect it to the Rust backend, create a `.env` file from `.env.example` and fill in:

```bash
REACT_APP_API_BASE_URL=https://your-api-id.execute-api.your-region.amazonaws.com/your-stage
REACT_APP_EVENT_ID=evt_your_event_id
```

Restart `npm start` after changing `.env`.

### Backend Project

The backend from `https://github.com/91tleg/blc.git` is included at the repository root.

```bash
npm run backend:test
npm run backend:build
```

## Usage

### Submitting a Signup

1. Fill out the form with:
   - First Name (optional)
   - Last Name (optional)
   - **At least one name field is required**
   
2. Select country code for phone number (default: +1 USA)

3. Enter phone number (required)

4. Enter email (required) or modify the domain

5. Click "Save" button

6. Your entry will appear in the Overview table below

### Viewing Records

- All submitted signups appear in the "Overview" table
- Records are displayed with the date they were submitted
- Unavailable information is marked as "N/A"

### Filtering Records

1. Use the "From" date picker to set start date
2. Use the "To" date picker to set end date
3. Table automatically updates to show only records within the date range
4. Click "Clear Filters" to view all records

### Exporting Data

1. (Optionally) Set date filters first
2. Click "Export to CSV" button
3. A CSV file will download to your computer
4. File name format: `blc_records_YYYY-MM-DD.csv`

## Project Structure

```
BLC/
├── .github/
│   └── workflows/
├── docs/
├── scripts/
├── src/
├── web/
│   ├── public/
│   │   └── index.html
│   ├── src/
│   │   ├── components/
│   │   │   ├── Calendar.js
│   │   │   ├── EventPosters.js
│   │   │   ├── SignupForm.js
│   │   │   └── RecordsTable.js
│   │   ├── api.js
│   │   ├── App.js
│   │   ├── App.css
│   │   └── index.js
│   ├── blc_logo.png
│   ├── package.json
│   └── README.md
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## Validation Rules

### Form Requirements:
- **Name**: At least First Name OR Last Name required
- **Phone**: Required, minimum 7 digits
- **Email**: Required, must be valid email format

### Valid Examples:
- ✓ First Name: "John" + Phone + Email
- ✓ Last Name: "Doe" + Phone + Email
- ✓ First Name: "John", Last Name: "Doe" + Phone + Email

### Invalid Examples:
- ✗ No name fields filled
- ✗ Missing phone number
- ✗ Missing email address

## Features Overview

### Interactive Elements
- Form fields transform on focus with visual feedback
- Smooth animations and transitions
- Responsive design for mobile and desktop
- Background image integration (blc_logo.png)

### Data Persistence
- Records are saved to browser's LocalStorage when no backend is configured
- When backend env vars are configured, signups are submitted to the backend and mirrored locally for the on-page table/export
- Data persists even after browser closes

### Export Options
- Export to CSV format for spreadsheet applications
- Includes all visible fields: First Name, Last Name, Phone, Email, Date
- Filtered exports respect date range selections

## Browser Support

- Chrome (recommended)
- Firefox
- Safari
- Edge
- Any modern browser with LocalStorage support

## Troubleshooting

### "npm: command not found"
- Install Node.js from https://nodejs.org/
- Restart terminal after installation

### Port 3000 already in use
- The development server will prompt to use another port
- Or find and stop the process using port 3000

### Records not appearing
- Check browser's LocalStorage is enabled
- Open DevTools (F12) → Application → LocalStorage
- Look for 'blcRecords' key

## Contact & Support

For issues or questions about the Business Leadership Community:
Visit: https://linktr.ee/bc_blc

## License

© 2026 Bellevue College - Business Leadership Community
