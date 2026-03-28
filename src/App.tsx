import { Routes, Route } from 'react-router-dom';
import { LocaleProvider } from '@/providers/LocaleProvider';
import { TauriProvider } from '@/providers/TauriProvider';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { SettingsProvider } from '@/contexts/SettingsContext';
import { ToastProvider } from '@/contexts/ToastContext';
import ClientOnlyApp from '@/components/ClientOnlyApp';
import SettingsPage from '@/pages/SettingsPage';
import Shell from '@/blueprint/layout/Shell';
import ManuscriptTest from '@/pages/ManuscriptTest';
import DashboardPanel from '@/blueprint/Dashboard/DashboardPanel';

function AppProviders({ children }: { children: React.ReactNode }) {
  return (
    <TauriProvider>
      <ThemeProvider>
        <SettingsProvider>
          <LocaleProvider>
            <ToastProvider>
              {children}
            </ToastProvider>
          </LocaleProvider>
        </SettingsProvider>
      </ThemeProvider>
    </TauriProvider>
  );
}

export default function App() {
  return (
    <AppProviders>
      <Routes>
        <Route path="/" element={<ClientOnlyApp />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="/prototype" element={<Shell />} />
        <Route path="/manuscript-test" element={<ManuscriptTest />} />
        <Route path="/dashboard" element={<DashboardPanel />} />
      </Routes>
    </AppProviders>
  );
}
