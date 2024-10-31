import { ErrorBoundary, FallbackProps } from "react-error-boundary";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import ClusterOverview from "./pages/ClusterOverview";
import ClusterView from "./pages/ClusterView";
import { emit } from "@tauri-apps/api/event";

import classes from './AppShell.module.css';
import StatusPanel from "./containers/StatusPanel";

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

const router = createBrowserRouter([
    {
        path: "/",
        element: <ClusterOverview />,
        errorElement: <p>Not found 🥲</p>
    },
    {
        path: "/cluster",
        element: <ClusterView />,
        errorElement: <p>Not found 🥲</p>
    },
]);

const AppShell: React.FC = () => {
    return (
        <ErrorBoundary fallbackRender={fallbackRender}>
            <div className={classes.container}>
                <main className={classes.main}>
                    <RouterProvider router={router} />
                </main>
                <footer className={classes.footer}>
                    <StatusPanel />
                </footer>
            </div>
        </ErrorBoundary>
    );
};

export default AppShell;
