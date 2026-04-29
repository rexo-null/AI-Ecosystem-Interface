import React, { useState, useEffect } from 'react';
import './UpdatePanel.css';

interface UpdateStatus {
  available: boolean;
  version: string | null;
  release_notes: string | null;
  download_url: string | null;
}

export const UpdatePanel: React.FC = () => {
  const [checking, setChecking] = useState(false);
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    // Check for updates on mount
    checkForUpdates();
  }, []);
  
  const checkForUpdates = async () => {
    setChecking(true);
    setError(null);
    try {
      // Simulated update check - in real app would call Tauri command
      await new Promise(resolve => setTimeout(resolve, 1000));
      setUpdateStatus({
        available: false,
        version: '0.1.0-alpha',
        release_notes: null,
        download_url: null,
      });
    } catch (err) {
      setError('Failed to check for updates');
    } finally {
      setChecking(false);
    }
  };
  
  const downloadUpdate = async () => {
    if (!updateStatus?.download_url) return;
    
    setDownloading(true);
    setDownloadProgress(0);
    
    // Simulate download progress
    for (let i = 0; i <= 100; i += 10) {
      await new Promise(resolve => setTimeout(resolve, 200));
      setDownloadProgress(i);
    }
    
    setDownloading(false);
  };
  
  const installUpdate = async () => {
    // Would call Tauri command to install update
    window.location.reload();
  };
  
  return (
    <div className="update-panel">
      <div className="panel-header">
        <h3>Updates</h3>
        <span className="version">v{updateStatus?.version || '...'}</span>
      </div>
      
      <div className="panel-content">
        {error && (
          <div className="error-message">{error}</div>
        )}
        
        {!updateStatus ? (
          <div className="checking">Checking for updates...</div>
        ) : updateStatus.available ? (
          <div className="update-available">
            <div className="update-info">
              <span className="update-icon">🎉</span>
              <p>Version {updateStatus.version} is available!</p>
            </div>
            
            {updateStatus.release_notes && (
              <div className="release-notes">
                <h4>Release Notes</h4>
                <p>{updateStatus.release_notes}</p>
              </div>
            )}
            
            {downloading ? (
              <div className="download-progress">
                <div className="progress-bar">
                  <div 
                    className="progress-fill"
                    style={{ width: `${downloadProgress}%` }}
                  />
                </div>
                <span className="progress-text">{downloadProgress}%</span>
              </div>
            ) : (
              <div className="update-actions">
                <button 
                  className="btn btn-download"
                  onClick={downloadUpdate}
                >
                  Download Update
                </button>
                <button 
                  className="btn btn-install"
                  onClick={installUpdate}
                >
                  Install & Restart
                </button>
              </div>
            )}
          </div>
        ) : (
          <div className="up-to-date">
            <span className="check-icon">✅</span>
            <p>You're up to date!</p>
            <button 
              className="btn btn-check"
              onClick={checkForUpdates}
              disabled={checking}
            >
              {checking ? 'Checking...' : 'Check Again'}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default UpdatePanel;