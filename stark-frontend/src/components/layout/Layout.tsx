import { Outlet, Navigate } from 'react-router-dom';
import Sidebar from './Sidebar';
import BottomNav from './BottomNav';
import DiskQuotaBanner from './DiskQuotaBanner';
import { useAuth } from '@/hooks/useAuth';
import UnicodeSpinner from '@/components/ui/UnicodeSpinner';

export default function Layout() {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="flex items-center gap-3">
          <UnicodeSpinner animation="rain" size="lg" className="text-stark-500" />
          <span className="text-slate-400">Loading...</span>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <Navigate to="/" replace />;
  }

  return (
    <div className="h-screen flex overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-y-auto pb-16 md:pb-0">
        <DiskQuotaBanner />
        <Outlet />
      </main>
      <BottomNav />
    </div>
  );
}
