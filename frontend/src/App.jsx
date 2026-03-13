// Nexus CMS - Main App
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './context/AuthContext';
import { Layout } from './components/Layout';
import { LoginPage } from './pages/LoginPage';
import { PagesListPage } from './pages/PagesListPage';
import { PageViewPage } from './pages/PageViewPage';
import { PageEditorPage } from './pages/PageEditorPage';
import { AdminPage } from './pages/AdminPage';

// Protected Route wrapper
function ProtectedRoute({ children }) {
  const { user, loading } = useAuth();
  
  if (loading) return <div className="loading">Loading...</div>;
  if (!user) return <Navigate to="/login" replace />;
  
  return children;
}

// Public Route (redirect if logged in)
function PublicRoute({ children }) {
  const { user, loading } = useAuth();
  
  if (loading) return <div className="loading">Loading...</div>;
  if (user) return <Navigate to="/" replace />;
  
  return children;
}

function AppRoutes() {
  return (
    <Routes>
      {/* Public Routes */}
      <Route path="/" element={<Layout><PagesListPage /></Layout>} />
      <Route path="/page/:slug" element={<Layout><PageViewPage /></Layout>} />
      <Route path="/login" element={<PublicRoute><LoginPage /></PublicRoute>} />
      
      {/* Protected Routes */}
      <Route path="/admin" element={<ProtectedRoute><Layout><AdminPage /></Layout></ProtectedRoute>} />
      <Route path="/admin/page/:slug" element={<ProtectedRoute><Layout><PageEditorPage /></Layout></ProtectedRoute>} />
      
      {/* 404 */}
      <Route path="*" element={<Layout><div className="not-found"><h1>404</h1><p>Page not found</p></div></Layout>} />
    </Routes>
  );
}

function App() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <AppRoutes />
      </AuthProvider>
    </BrowserRouter>
  );
}

export default App;
