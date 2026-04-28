import { Outlet } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { StatusBar } from './StatusBar'

export default function Layout() {
  return (
    <div className="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-950 text-black dark:text-white">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        <main className="flex-1 overflow-y-auto p-8 scrollbar-hide">
          <Outlet />
        </main>
        <StatusBar />
      </div>
    </div>
  )
}
