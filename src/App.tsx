import { Routes, Route } from 'react-router-dom';
import { LocaleProvider } from '@/providers/LocaleProvider';
import { TauriProvider } from '@/providers/TauriProvider';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { SettingsProvider } from '@/contexts/SettingsContext';
import { ToastProvider } from '@/contexts/ToastContext';
import Shell from '@/blueprint/layout/Shell';
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
        <Route path="/" element={<Shell />} />
        <Route path="/dashboard" element={<DashboardPanel />} />
      </Routes>
    </AppProviders>
  );
}
