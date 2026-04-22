import React, { useRef, useState } from 'react';

const MAX_IMAGE_DIMENSION = 1400;
const IMAGE_QUALITY = 0.85;

const createPosterId = () => {
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
};

const readFileAsDataUrl = (file) => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(new Error('Unable to read image file'));
    reader.readAsDataURL(file);
  });
};

const loadImage = (dataUrl) => {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error('Unable to load image file'));
    image.src = dataUrl;
  });
};

const fileToPoster = async (file) => {
  const sourceDataUrl = await readFileAsDataUrl(file);
  const image = await loadImage(sourceDataUrl);
  const scale = Math.min(1, MAX_IMAGE_DIMENSION / Math.max(image.width, image.height));
  const width = Math.max(1, Math.round(image.width * scale));
  const height = Math.max(1, Math.round(image.height * scale));
  const canvas = document.createElement('canvas');
  const context = canvas.getContext('2d');

  canvas.width = width;
  canvas.height = height;
  context.fillStyle = '#ffffff';
  context.fillRect(0, 0, width, height);
  context.drawImage(image, 0, 0, width, height);

  return {
    id: createPosterId(),
    name: file.name,
    dataUrl: canvas.toDataURL('image/jpeg', IMAGE_QUALITY),
    uploadedAt: new Date().toISOString()
  };
};

const EventPosters = ({
  selectedDateLabel,
  posters,
  onAddPosters,
  onReplacePoster,
  onRemovePoster,
  storageError
}) => {
  const addInputRef = useRef(null);
  const replaceInputRefs = useRef({});
  const [error, setError] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);

  const processFiles = async (files, onComplete) => {
    const selectedFiles = Array.from(files || []);
    if (selectedFiles.length === 0) {
      return;
    }

    const imageFiles = selectedFiles.filter(file => file.type.startsWith('image/'));

    if (imageFiles.length === 0) {
      setError('Choose an image file for the event poster.');
      return;
    }

    setIsProcessing(true);
    setError('');

    try {
      const nextPosters = await Promise.all(imageFiles.map(fileToPoster));
      onComplete(nextPosters);
    } catch {
      setError('That poster could not be uploaded. Try a JPG or PNG file.');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleAddFiles = async (event) => {
    await processFiles(event.target.files, onAddPosters);
    event.target.value = '';
  };

  const handleReplaceFile = async (event, posterId) => {
    await processFiles(event.target.files, ([poster]) => {
      onReplacePoster(posterId, poster);
    });
    event.target.value = '';
  };

  return (
    <section className="event-posters">
      <div className="event-posters-header">
        <div>
          <h3>Event Poster</h3>
          <p>{selectedDateLabel}</p>
        </div>
        <button
          type="button"
          className="btn-event-upload"
          onClick={() => addInputRef.current?.click()}
          disabled={isProcessing}
        >
          {posters.length > 0 ? 'Add Another Poster' : 'Add Event Poster'}
        </button>
        <input
          ref={addInputRef}
          type="file"
          accept="image/*"
          multiple
          className="file-input-hidden"
          onChange={handleAddFiles}
        />
      </div>

      {(error || storageError) && (
        <div className="event-posters-error">
          {error || storageError}
        </div>
      )}

      {posters.length === 0 ? (
        <div className="no-event-posters">
          No event poster uploaded for this date.
        </div>
      ) : (
        <div className="event-poster-grid">
          {posters.map(poster => (
            <article className="event-poster-card" key={poster.id}>
              <img src={poster.dataUrl} alt={poster.name} />
              <div className="event-poster-actions">
                <button
                  type="button"
                  className="btn-poster-change"
                  onClick={() => replaceInputRefs.current[poster.id]?.click()}
                  disabled={isProcessing}
                >
                  Change
                </button>
                <button
                  type="button"
                  className="btn-poster-remove"
                  onClick={() => onRemovePoster(poster.id)}
                  disabled={isProcessing}
                >
                  Remove
                </button>
                <input
                  ref={input => {
                    replaceInputRefs.current[poster.id] = input;
                  }}
                  type="file"
                  accept="image/*"
                  className="file-input-hidden"
                  onChange={(event) => handleReplaceFile(event, poster.id)}
                />
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
};

export default EventPosters;
