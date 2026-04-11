import { Routes, Route } from 'react-router-dom';
import { LocaleProvider } from '@/providers/LocaleProvider';
import { TauriProvider } from '@/providers/TauriProvider';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { ToastProvider } from '@/contexts/ToastContext';
import Shell from '@/components/layout/Shell';

function AppProviders({ children }: { children: React.ReactNode }) {
  return (
    <TauriProvider>
      <ThemeProvider>
        <LocaleProvider>
          <ToastProvider>
            {children}
          </ToastProvider>
        </LocaleProvider>
      </ThemeProvider>
    </TauriProvider>
  );
}

export default function App() {
  return (
    <AppProviders>
      <Routes>
        <Route path="/" element={<Shell />} />
      </Routes>
    </AppProviders>
  );
}
