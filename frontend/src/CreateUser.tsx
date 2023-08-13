import { useContext, useState } from "react";
import { AuthContext } from "./Auth";
import { useLocation, useNavigate } from "react-router-dom";

function CreateUser() {
    const [userName, setUserName] = useState("");
    const [password, setPassword] = useState("");
    const auth = useContext(AuthContext);
    const navigate = useNavigate();
    const location = useLocation();

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        if (userName === "") {
            alert("Invalid username");
            return;
        }

        if (password === "") {
            alert("Invalid password");
            return;
        }

        auth.createUser(userName, password, (success) => {
            if (success) {
                auth.login(userName, password, (success) => {
                    if (success) {
                        navigate("/", {
                            replace: true,
                            state: { from: location },
                        });
                    } else {
                        alert("Failed to log in after user creation");
                    }
                });
            } else {
                alert("Unable to create account");
            }
        });
    };

    if (auth.getUser()) {
        return <div>You are already logged in</div>;
    }

    return (
        <div>
            <h1>Create User</h1>
            <form onSubmit={handleSubmit}>
                <input
                    type="text"
                    placeholder="username"
                    value={userName}
                    onChange={(e) => setUserName(e.target.value)}
                />
                <input
                    type="password"
                    placeholder="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                />
                <input type="submit" />
            </form>
        </div>
    );
}

export default CreateUser;
