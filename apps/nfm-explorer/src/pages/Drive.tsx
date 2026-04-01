import { HardDrive, Upload, Folder, File, Shield, Database, ArrowRight } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAppData } from '../context/AppDataContext';
import { appDriveDownload, appDriveUpload } from '../api/client';

const Drive = () => {
  const navigate = useNavigate();
  const { data, notifyError, notifySuccess, requestPrompt, refresh } = useAppData();
  const DUMMY_DRIVE_FILES = data.drive_files;
  const totalFragments = DUMMY_DRIVE_FILES.reduce((sum, file) => sum + file.fragments, 0);
  const averageHealth = DUMMY_DRIVE_FILES.length > 0
    ? Math.round(DUMMY_DRIVE_FILES.reduce((sum, file) => sum + file.health, 0) / DUMMY_DRIVE_FILES.length)
    : 0;

  const handleUpload = async () => {
    const name = await requestPrompt({
      title: 'Upload File',
      message: 'Enter file name',
      placeholder: 'my-notes.txt',
      confirmText: 'Upload',
    });

    if (!name || !name.trim()) {
      return;
    }

    const content = await requestPrompt({
      title: 'File Content',
      message: 'Paste short file content (indexed as preview)',
      placeholder: 'Type content here...',
      confirmText: 'Submit',
    });

    if (content === null) {
      return;
    }

    try {
      await appDriveUpload(name.trim(), content, data.user_profile.nfmAddress);
      await refresh();
      notifySuccess('Drive file uploaded');
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Upload failed';
      notifyError(message);
    }
  };

  const handleViewAllVaultFiles = () => {
    sessionStorage.setItem('nfm_explorer_query', '/root/my-vault');
    navigate('/explorer');
  };

  const handleDownload = async (fileId: string, fileName: string) => {
    try {
      const result = await appDriveDownload(fileId, data.user_profile.nfmAddress) as { content?: string; file?: { content_preview?: string } };
      const content = result.content || result.file?.content_preview || '';

      if (content && navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(content);
        notifySuccess(`Downloaded ${fileName}. Content copied to clipboard.`);
      } else {
        notifySuccess(`Downloaded ${fileName}.`);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Download failed';
      notifyError(message);
    }
  };

  return (
    <div className="animate-in">
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-cyan flex items-center gap-2"><HardDrive /> NFM Distributed Drive</h1>
        <div className="flex gap-4 items-center">
          <span className="text-muted text-sm font-mono hide-mobile">{DUMMY_DRIVE_FILES.length} files indexed</span>
          <button className="nfm-btn nfm-btn--primary nfm-btn--sm" onClick={handleUpload}>
            <Upload size={14} /> Upload File
          </button>
        </div>
      </div>

      <div className="flex gap-6 flex-wrap">
        
        {/* Drive Storage Stats */}
        <div className="nfm-glass-card nfm-glass-card--glow-cyan" style={{ flex: '1 1 320px' }}>
          <div className="flex items-center gap-2 mb-6">
            <Database className="text-cyan" size={20} />
            <h2 className="text-lg">Storage Health</h2>
          </div>
          
          <div className="flex-col gap-4">
            <div className="nfm-storage-item">
               <span className="nfm-storage-item__label">Total Network Fragments</span>
               <div className="flex justify-between items-end">
                 <span className="nfm-storage-item__value">{totalFragments.toLocaleString()}</span>
                 <span className="text-muted text-xs mb-1">{DUMMY_DRIVE_FILES.length} Vault Files</span>
               </div>
            </div>

            <div className="nfm-storage-item flex items-center gap-4">
               <Shield className="text-success" size={24} />
               <div>
                 <div className="font-bold text-sm">ZKP Encrypted</div>
                 <div className="text-muted text-xs">Bio-ZKP Decryption Only</div>
               </div>
            </div>
            
            <div className="mt-2">
              <div className="flex justify-between text-xs mb-2">
                <span className="text-muted">Fragment Health</span>
                <span className="text-cyan font-mono">{averageHealth}%</span>
              </div>
              <div className="nfm-progress" style={{ height: '6px' }}>
                <div className="nfm-progress__fill nfm-progress__fill--cyan" style={{ width: `${averageHealth}%` }}></div>
              </div>
              <div className="flex justify-between text-[10px] text-muted mt-2 uppercase tracking-wider">
                <span>degraded</span>
                <span>healthy</span>
              </div>
            </div>
          </div>
        </div>

        {/* File Browser */}
        <div className="nfm-glass-card" style={{ flex: '2 1 600px' }}>
          <div className="flex items-center gap-2 mb-6">
            <Folder className="text-purple" size={20} />
            <h2 className="text-lg">/root/my-vault</h2>
          </div>

          <table className="nfm-table">
            <thead>
              <tr>
                <th>File Name</th>
                <th>Size</th>
                <th>Health</th>
                <th>Fragments</th>
                <th>Uploaded</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {DUMMY_DRIVE_FILES.map(file => (
                <tr key={file.id}>
                  <td className="font-medium flex items-center gap-3">
                    <File className="text-cyan" size={14} /> {file.name}
                  </td>
                  <td className="font-mono text-xs text-secondary">{file.size}</td>
                  <td>
                    <div className="flex items-center gap-2">
                      <div className="nfm-progress" style={{ width: '60px', height: '4px' }}>
                        <div className={`nfm-progress__fill nfm-progress__fill--${file.health === 100 ? 'success' : 'warning'}`} style={{ width: `${file.health}%` }}></div>
                      </div>
                      <span className={`text-[10px] font-bold ${file.health === 100 ? 'text-success' : 'text-warning'}`}>{file.health}%</span>
                    </div>
                  </td>
                  <td className="font-mono text-xs">{file.fragments}</td>
                  <td className="text-muted text-xs">{Math.floor((Date.now() - file.uploadedAt) / 3600000)}h ago</td>
                  <td>
                    <button
                      className="nfm-btn nfm-btn--ghost nfm-btn--sm"
                      onClick={() => void handleDownload(file.id, file.name)}
                    >
                      Download
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          
          <button className="nfm-btn-more" onClick={handleViewAllVaultFiles}>
            <ArrowRight size={14} /> View All Vault Files
          </button>

          <div className="nfm-dropzone">
             <p className="nfm-dropzone__text">Drag & Drop files here, or click to browse.</p>
             <p className="nfm-dropzone__hint">Files are automatically chunked, encrypted, and distributed.</p>
          </div>
        </div>

      </div>
    </div>
  );
};

export default Drive;
