import { BrowserRouter, Routes, Route } from "react-router-dom";
import Login from "./Login";
import Home from "./Home";
import Logout from "./Logout";
import { AuthProtected, AuthProvider } from "./Auth";
import PracticeSessions from "./PracticeSessions";
import Pieces from "./Pieces";
import CreateUser from "./CreateUser";

function App() {
    return (
        <AuthProvider>
            <BrowserRouter>
                <Routes>
                    <Route path="/login" Component={Login} />
                    <Route path="/logout" Component={Logout} />
                    <Route path="/create-user" Component={CreateUser} />
                    <Route
                        path="/practice-sessions"
                        element={
                            <AuthProtected>
                                <PracticeSessions />
                            </AuthProtected>
                        }
                    />
                    <Route
                        path="/pieces"
                        element={
                            <AuthProtected>
                                <Pieces />
                            </AuthProtected>
                        }
                    />
                    <Route path="/" Component={Home} />
                </Routes>
            </BrowserRouter>
        </AuthProvider>
    );
}

export default App;
