import { ErrorBoundary, FallbackProps } from "react-error-boundary";
import { createBrowserRouter, Outlet, RouterProvider } from "react-router-dom";
import ClusterOverview from "./pages/ClusterOverview";
import ClusterView from "./pages/ClusterView";
import { emit } from "@tauri-apps/api/event";

import classes from './AppShell.module.css';
import StatusPanel from "./containers/StatusPanel";
import NavHeader from "./components/NavHeader";

function fallbackRender(context: FallbackProps) {
    // Call resetErrorBoundary() to reset the error boundary and retry the render.

    return (
        <div role="alert">
            <p>Something went wrong:</p>
            <pre style={{ color: "red" }}>{context.error.message}</pre>
        </div>
    );
}

window.onbeforeunload = function () {
    emit('frontend-onbeforeunload');
};

const Layout: React.FC = () => {
    return (
        <ErrorBoundary fallbackRender={fallbackRender}>
            <div className={classes.container}>
                <header className={classes.header}>
                    <NavHeader />
                </header>
                <main className={classes.main}>
                    <ErrorBoundary fallbackRender={fallbackRender}>
                        <Outlet />
                    </ErrorBoundary>
                </main>
                <footer className={classes.footer}>
                    <StatusPanel />
                </footer>
            </div>
        </ErrorBoundary>
    );
};

const router = createBrowserRouter([
    {
        element: <Layout />,
        children: [
            {
                path: "/",
                element: <ClusterOverview />,
                errorElement: <p>ClusterOverview: Not found 🥲</p>,
            },
            {
                path: "cluster",
                element: <ClusterView />,
                errorElement: <p>ClusterView: Not found 🥲</p>
            }
        ]
    }
]);

const AppShell: React.FC = () => {
    return <RouterProvider router={router} />;
};

export default AppShell;
