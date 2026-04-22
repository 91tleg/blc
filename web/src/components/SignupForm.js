import React, { useState } from 'react';

const SignupForm = ({ onSave, isRotating, selectedDateLabel, backendEnabled = false }) => {
  const [formData, setFormData] = useState({
    firstName: '',
    lastName: '',
    countryCode: '+1',
    phone: '',
    emailLocalPart: '',
    emailDomain: 'bellevuecollege.edu'
  });

  const [errors, setErrors] = useState([]);
  const [successMessage, setSuccessMessage] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const countryCodes = [
    { code: '+1', country: 'US' },
    { code: '+44', country: 'UK' },
    { code: '+1', country: 'Canada' },
    { code: '+91', country: 'India' },
    { code: '+86', country: 'China' },
    { code: '+81', country: 'Japan' },
    { code: '+33', country: 'France' },
    { code: '+49', country: 'Germany' },
    { code: '+39', country: 'Italy' },
    { code: '+34', country: 'Spain' },
    { code: '+61', country: 'Australia' },
    { code: '+64', country: 'New Zealand' }
  ];

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: value
    }));
    if (errors.length > 0) {
      setErrors([]);
    }
  };

  const validateForm = () => {
    const newErrors = [];
    const firstName = formData.firstName.trim();
    const lastName = formData.lastName.trim();
    const phone = formData.phone.trim();
    const emailLocalPart = formData.emailLocalPart.trim();
    const emailDomain = formData.emailDomain.trim();

    const hasName = firstName || lastName;
    const hasPhone = phone.length > 0;
    const hasEmail = emailLocalPart.length > 0;

    if (!hasName) {
      newErrors.push('At least First Name or Last Name is required');
    }

    if (backendEnabled && !hasEmail) {
      newErrors.push('Email address is required for backend signups');
    } else if (!hasPhone && !hasEmail) {
      newErrors.push('Enter either a phone number or an email address');
    }

    if (hasPhone) {
      const phoneDigits = phone.replace(/\D/g, '');
      const minimumDigits = backendEnabled ? 10 : 7;
      if (phoneDigits.length < minimumDigits) {
        newErrors.push(`Phone number must be at least ${minimumDigits} digits`);
      } else if (backendEnabled && phoneDigits.length > 15) {
        newErrors.push('Phone number must be no more than 15 digits');
      }
    }

    if (hasEmail && !emailDomain) {
      newErrors.push('Email domain is required');
    } else if (hasEmail && !isValidEmail(`${emailLocalPart}@${emailDomain}`)) {
      newErrors.push('Please enter a valid email address');
    }

    return newErrors;
  };

  const isValidEmail = (email) => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  };

  const handleSave = async (e) => {
    e.preventDefault();
    const newErrors = validateForm();

    if (newErrors.length > 0) {
      setErrors(newErrors);
      setSuccessMessage('');
      return;
    }

    const emailLocalPart = formData.emailLocalPart.trim();
    const emailDomain = formData.emailDomain.trim();
    const phoneNumber = formData.phone.trim();
    const email = emailLocalPart ? `${emailLocalPart}@${emailDomain}` : undefined;
    const phone = phoneNumber ? `${formData.countryCode} ${phoneNumber}` : undefined;

    setIsSubmitting(true);

    try {
      await onSave({
        firstName: formData.firstName.trim() || undefined,
        lastName: formData.lastName.trim() || undefined,
        phone: phone,
        email: email
      });

      setSuccessMessage('✓ Signup saved successfully!');
      setErrors([]);

      setFormData({
        firstName: '',
        lastName: '',
        countryCode: '+1',
        phone: '',
        emailLocalPart: '',
        emailDomain: 'bellevuecollege.edu'
      });

      setTimeout(() => {
        setSuccessMessage('');
      }, 3000);
    } catch (error) {
      setErrors([error.message || 'Unable to save signup. Try again.']);
      setSuccessMessage('');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleReset = (e) => {
    e.preventDefault();
    setFormData({
      firstName: '',
      lastName: '',
      countryCode: '+1',
      phone: '',
      emailLocalPart: '',
      emailDomain: 'bellevuecollege.edu'
    });
    setErrors([]);
    setSuccessMessage('');
  };

  return (
    <div className="signup-section">
      {/* LEFT SIDEBAR HEADER */}
      <div className="signup-header">
        <div className="header-icon">📋</div>
        <h1>Sign Up</h1>
        <p>Join BLC Today</p>
        <p style={{ fontSize: '12px', opacity: 0.8, marginTop: '15px' }}>
          Business Leadership Community
        </p>
      </div>

      {/* RIGHT FORM CONTENT */}
      <div className={`form-content ${isRotating ? 'rotating' : ''}`}>
        <form onSubmit={handleSave}>
          <div className="signup-date-context">
            Saving signups for {selectedDateLabel}
          </div>

          {/* Error Messages */}
          {errors.length > 0 && (
            <div style={{ 
              background: 'rgba(255, 235, 238, 0.85)', 
              border: '2px solid #e74c3c', 
              borderRadius: '10px', 
              padding: '14px', 
              marginBottom: '22px', 
              color: '#c0392b',
              animation: 'slideInFromLeft 0.5s ease-out',
              fontWeight: '600'
            }}>
              {errors.map((error, idx) => (
                <div key={idx} style={{ marginBottom: idx < errors.length - 1 ? '6px' : '0' }}>
                  ✗ {error}
                </div>
              ))}
            </div>
          )}

          {/* Success Message */}
          {successMessage && (
            <div style={{ 
              background: 'rgba(232, 245, 233, 0.85)', 
              border: '2px solid #27ae60', 
              borderRadius: '10px', 
              padding: '14px', 
              marginBottom: '22px', 
              color: '#27ae60', 
              fontWeight: '700',
              animation: 'slideInFromLeft 0.5s ease-out'
            }}>
              {successMessage}
            </div>
          )}

          {/* Name Fields Row */}
          <div className="form-row">
            <div className="form-group">
              <label>First Name</label>
              <input
                type="text"
                name="firstName"
                value={formData.firstName}
                onChange={handleChange}
                placeholder="John"
              />
            </div>
            <div className="form-group">
              <label>Last Name</label>
              <input
                type="text"
                name="lastName"
                value={formData.lastName}
                onChange={handleChange}
                placeholder="Doe"
              />
            </div>
          </div>

          {/* Phone Number Row */}
          <div className="form-row">
            <div className="form-group">
              <label>Phone Number</label>
              <div className="phone-group">
                <select
                  name="countryCode"
                  value={formData.countryCode}
                  onChange={handleChange}
                  className="country-code"
                  style={{
                    padding: '14px',
                    border: '2px solid rgba(26, 84, 144, 0.15)',
                    borderRadius: '12px',
                    fontSize: '14px',
                    transition: 'all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275)',
                    cursor: 'pointer',
                    background: 'rgba(255, 255, 255, 0.72)',
                    fontWeight: '700',
                    color: '#1a1a1a'
                  }}
                  onFocus={(e) => {
                    e.target.style.borderColor = '#1a5490';
                    e.target.style.boxShadow = '0 0 25px rgba(26, 84, 144, 0.35)';
                    e.target.style.transform = 'translateY(-3px)';
                  }}
                  onBlur={(e) => {
                    e.target.style.borderColor = 'rgba(26, 84, 144, 0.15)';
                    e.target.style.boxShadow = 'none';
                    e.target.style.transform = 'translateY(0)';
                  }}
                >
                  {countryCodes.map((item) => (
                    <option key={`${item.code}-${item.country}`} value={item.code}>
                      {item.code} {item.country}
                    </option>
                  ))}
                </select>
                <input
                  type="tel"
                  name="phone"
                  value={formData.phone}
                  onChange={handleChange}
                  placeholder="(555) 123-4567"
                  className="phone-input"
                />
              </div>
            </div>
          </div>

          {/* Email Row */}
          <div className="form-row">
            <div className="form-group">
              <label>Email Address</label>
              <div className="email-group">
                <input
                  type="text"
                  name="emailLocalPart"
                  value={formData.emailLocalPart}
                  onChange={handleChange}
                  placeholder="student"
                  className="email-input"
                />
                <span className="email-at-symbol">@</span>
                <input
                  type="text"
                  name="emailDomain"
                  value={formData.emailDomain}
                  onChange={handleChange}
                  className="email-domain"
                />
              </div>
            </div>
          </div>

          {/* Buttons Row */}
          <div className="button-group">
            <button type="submit" className="btn-save" disabled={isSubmitting}>
              {isSubmitting ? 'Saving...' : '📤 Submit'}
            </button>
            <button type="reset" className="btn-reset" onClick={handleReset} disabled={isSubmitting}>✕ Reset</button>
          </div>

          {/* Link Section */}
          <div className="link-section">
            <p>🔗 Connect With Us</p>
            <a href="https://linktr.ee/bc_blc" target="_blank" rel="noopener noreferrer">
              linktr.ee/bc_blc
            </a>
          </div>
        </form>
      </div>
    </div>
  );
};

export default SignupForm;
