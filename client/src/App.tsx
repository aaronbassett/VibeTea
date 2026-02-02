import { useSessionTimeouts } from './hooks/useSessionTimeouts';

export default function App() {
  // Initialize session timeout checking at root level
  useSessionTimeouts();

  return (
    <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
      <h1 className="text-2xl font-bold">VibeTea Dashboard</h1>
    </div>
  );
}
