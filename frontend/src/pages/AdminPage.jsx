// Admin Dashboard Page
import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { admin, system } from '../api';
import { useAuth } from '../context/AuthContext';

export function AdminPage() {
  const { user, isAdmin } = useAuth();
  const [stats, setStats] = useState(null);
  const [users, setUsers] = useState([]);
  const [maintenance, setMaintenance] = useState(false);
  const [cvUrl, setCvUrl] = useState('');
  const [activeTab, setActiveTab] = useState('overview');

  useEffect(() => {
    if (isAdmin) {
      loadData();
    }
  }, [isAdmin]);

  const loadData = async () => {
    try {
      const [statsRes, usersRes, maintRes] = await Promise.all([
        admin.stats(),
        admin.users(),
        system.settings(),
      ]);
      setStats(statsRes.data);
      setUsers(usersRes.data);
      setMaintenance(maintRes.data.maintenance_mode);
      setCvUrl(maintRes.data.cv_url || '');
    } catch (err) {
      console.error('Failed to load admin data:', err);
    }
  };

  const toggleMaintenance = async () => {
    try {
      await system.updateSettings({ maintenance_mode: !maintenance });
      setMaintenance(!maintenance);
    } catch (err) {
      console.error('Failed to toggle maintenance:', err);
    }
  };

  const updateCvUrl = async () => {
    try {
      await system.updateSettings({ cv_url: cvUrl });
      alert('CV URL updated successfully!');
    } catch (err) {
      console.error('Failed to update CV URL:', err);
    }
  };

  const updateUserRole = async (userId, roleId) => {
    try {
      await admin.updateUser(userId, { role_id: roleId });
      loadData();
    } catch (err) {
      console.error('Failed to update user:', err);
    }
  };

  if (!isAdmin) {
    return (
      <div className="admin-unauthorized">
        <h2>Access Denied</h2>
        <p>You need admin privileges to access this page.</p>
      </div>
    );
  }

  return (
    <div className="admin-page">
      <div className="admin-header">
        <h1>Admin Dashboard</h1>
        <div className="admin-actions">
          <button 
            className={`btn ${maintenance ? 'btn-danger' : 'btn-warning'}`}
            onClick={toggleMaintenance}
          >
            {maintenance ? '🔧 Maintenance ON' : '✅ Maintenance OFF'}
          </button>
        </div>
      </div>

      <div className="admin-tabs">
        <button 
          className={activeTab === 'overview' ? 'active' : ''}
          onClick={() => setActiveTab('overview')}
        >Overview</button>
        <button 
          className={activeTab === 'users' ? 'active' : ''}
          onClick={() => setActiveTab('users')}
        >Users</button>
        <button 
          className={activeTab === 'system' ? 'active' : ''}
          onClick={() => setActiveTab('system')}
        >System</button>
      </div>

      {activeTab === 'overview' && stats && (
        <div className="stats-grid">
          <div className="stat-card">
            <span className="stat-icon">👥</span>
            <span className="stat-value">{stats.total_users}</span>
            <span className="stat-label">Users</span>
          </div>
          <div className="stat-card">
            <span className="stat-icon">📄</span>
            <span className="stat-value">{stats.total_pages}</span>
            <span className="stat-label">Pages</span>
          </div>
          <div className="stat-card">
            <span className="stat-icon">🧱</span>
            <span className="stat-value">{stats.total_blocks}</span>
            <span className="stat-label">Blocks</span>
          </div>
          <div className="stat-card">
            <span className="stat-icon">📁</span>
            <span className="stat-value">{stats.total_collections}</span>
            <span className="stat-label">Collections</span>
          </div>
          <div className="stat-card">
            <span className="stat-icon">🖼️</span>
            <span className="stat-value">{stats.total_media}</span>
            <span className="stat-label">Media</span>
          </div>
        </div>
      )}

      {activeTab === 'users' && (
        <div className="users-section">
          <h2>Users</h2>
          <table className="data-table">
            <thead>
              <tr>
                <th>Email</th>
                <th>Role</th>
                <th>Status</th>
                <th>Last Login</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {users.map(u => (
                <tr key={u.id}>
                  <td>{u.email}</td>
                  <td>
                    <select
                      value={u.role_id}
                      onChange={(e) => updateUserRole(u.id, e.target.value)}
                    >
                      <option value="00000000-0000-0000-0000-000000000001">Super-Architecte</option>
                      <option value="00000000-0000-0000-0000-000000000002">Gestionnaire</option>
                      <option value="00000000-0000-0000-0000-000000000003">VIP</option>
                      <option value="00000000-0000-0000-0000-000000000004">Visiteur</option>
                    </select>
                  </td>
                  <td>
                    <span className={`status ${u.is_active ? 'active' : 'inactive'}`}>
                      {u.is_active ? 'Active' : 'Inactive'}
                    </span>
                  </td>
                  <td>{u.last_login ? new Date(u.last_login).toLocaleDateString() : 'Never'}</td>
                  <td>
                    <button 
                      className="btn-sm"
                      onClick={() => updateUserRole(u.id, u.is_active ? 'disable' : 'enable')}
                    >
                      {u.is_active ? 'Disable' : 'Enable'}
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {activeTab === 'system' && (
        <div className="system-section">
          <h2>System Settings</h2>
          <div className="settings-group" style={{ marginBottom: '30px' }}>
            <h3>CV / Resume</h3>
            <p>Enter the URL of your CV (e.g. from the Media Library) so visitors can download it.</p>
            <div style={{ display: 'flex', gap: '10px', marginTop: '10px' }}>
              <input 
                type="text" 
                value={cvUrl} 
                onChange={(e) => setCvUrl(e.target.value)} 
                placeholder="/uploads/my-cv.pdf" 
                style={{ flex: 1, padding: '8px' }}
              />
              <button className="btn btn-primary" onClick={updateCvUrl}>Save</button>
            </div>
          </div>

          <h2>Quick Links</h2>
          <div className="quick-links">
            <Link to="/admin/page/new" className="quick-link-card">
              <span className="icon">📄</span>
              <span>New Page</span>
            </Link>
            <Link to="/admin/media" className="quick-link-card">
              <span className="icon">🖼️</span>
              <span>Media Library</span>
            </Link>
            <Link to="/admin/collections" className="quick-link-card">
              <span className="icon">📁</span>
              <span>Collections</span>
            </Link>
          </div>
        </div>
      )}
    </div>
  );
}
