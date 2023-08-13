import { createContext, useContext } from "react";
import { Navigate, useLocation } from "react-router-dom";
import { getRootURL } from "./fetch";

interface User {
    userId: number;
    userName: string;
}

interface AuthContextType {
    getUser: () => User | null;
    login: (
        userName: string,
        password: string,
        callback: (loginSuccess: boolean) => void
    ) => void;
    logout: (callback: (logoutSuceess: boolean) => void) => void;
}

const AuthContext = createContext<AuthContextType>(null!);

const setUser = (user: User) =>
    localStorage.setItem("user", JSON.stringify(user));

const getUser = () => {
    let stringifiedUser = localStorage.getItem("user");
    if (!stringifiedUser) {
        return null;
    }
    return JSON.parse(stringifiedUser);
};

const removeUser = () => localStorage.removeItem("user");

const login = (
    userName: string,
    password: string,
    callback: (loginSuccess: boolean) => void
) => {
    fetch(getRootURL() + "/api/login", {
        mode: "cors",
        credentials: "include",
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            user_name: userName,
            password: password,
        }),
    })
        .then((res) => res.json())
        .then((content) => {
            if (content.success) {
                setUser({
                    userId: content.user_id,
                    userName: content.user_name,
                });
            }
            callback(content.success);
        });
};

const logout = (callback: (logoutSuccess: boolean) => void) => {
    fetch(getRootURL() + "/api/logout", {
        mode: "cors",
        credentials: "include",
    })
        .then((res) => res.json())
        .then((content) => {
            if (content.success) {
                removeUser();
            }
            callback(content.success);
        });
};

const AuthProvider = ({ children }: { children: React.ReactNode }) => {
    return (
        <AuthContext.Provider value={{ getUser, login, logout }}>
            {children}
        </AuthContext.Provider>
    );
};

function AuthProtected({ children }: { children: React.JSX.Element }) {
    const auth = useContext(AuthContext);
    const location = useLocation();

    if (!auth.getUser()) {
        return (
            <Navigate to="/login" replace={true} state={{ from: location }} />
        );
    }

    return children;
}

export { AuthContext, AuthProtected, AuthProvider };
