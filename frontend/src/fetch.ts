import { Piece, PiecePracticedMapping, PracticeSession } from "./api-types";

type ErrorHandler = (error: string) => void;

type FetchFn<ResponseDataType> = (
    successCallback: (responseData: ResponseDataType) => void,
    errorCallback: ErrorHandler
) => Promise<void>;

type PostFn<InputDataType, ResponseDataType> = (
    entry: InputDataType,
    successCallback: (responseData: ResponseDataType) => void,
    errorCallback: ErrorHandler
) => Promise<void>;

const getRootURL = (): string => {
    const env = process.env.NODE_ENV;
    if (env === "production") {
        return process.env.REACT_APP_API_URL || "";
    } else {
        return "";
    }
};

const fetchPieces: FetchFn<Piece[]> = async (
    successCallback,
    errorCallback
) => {
    let res = await fetch(getRootURL() + "/api/get_pieces", {
        mode: "cors",
        credentials: "include",
    });
    let content = await res.json();

    if (content.success) {
        successCallback(content.pieces);
    } else {
        errorCallback("Error fetching pieces: " + content.error);
    }
};

const addPiece: PostFn<{ composer: string; title: string }, Piece> = async (
    piece,
    successCallback,
    errorCallback
) => {
    let res = await fetch(getRootURL() + "/api/create_piece", {
        mode: "cors",
        credentials: "include",
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(piece),
    });

    let content = await res.json();

    if (content.success) {
        successCallback(content.piece);
    } else {
        errorCallback("Failed to add piece: " + content.error);
    }
};

const fetchPracticeSessions: FetchFn<PracticeSession[]> = async (
    successCallback,
    errorCallback
) => {
    let res = await fetch(getRootURL() + "/api/get_practice_sessions", {
        mode: "cors",
        credentials: "include",
    });
    let content = await res.json();
    if (content.success) {
        successCallback(content.practice_sessions);
    } else {
        errorCallback("Error fetching practice sessions: " + content.error);
    }
};

const addPracticeSession: PostFn<
    {
        startDatetime: string;
        durationMins: number;
        instrument: string;
    },
    PracticeSession
> = async (practiceSession, successCallback, errorCallback) => {
    let res = await fetch(getRootURL() + "/api/create_practice_session", {
        mode: "cors",
        credentials: "include",
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            start_datetime: practiceSession.startDatetime + ":00",
            duration_mins: practiceSession.durationMins,
            instrument: practiceSession.instrument,
        }),
    });

    let content = await res.json();
    if (content.success) {
        successCallback(content.practice_session);
    } else {
        errorCallback("Failed to add practice session: " + content.error);
    }
};

const addPiecePracticed: PostFn<
    {
        practiceSessionId: number;
        pieceId: number;
    },
    PiecePracticedMapping
> = async (piece_practiced_mapping, successCallback, errorCallback) => {
    let res = await fetch(getRootURL() + "/api/create_piece_practiced", {
        mode: "cors",
        credentials: "include",
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            practice_session_id: piece_practiced_mapping.practiceSessionId,
            piece_id: piece_practiced_mapping.pieceId,
        }),
    });

    let content = await res.json();
    if (content.success) {
        successCallback(content.piece_practiced);
    } else {
        errorCallback(
            "Failed to add piece practiced mapping: " + content.error
        );
    }
};

export {
    fetchPieces,
    addPiece,
    fetchPracticeSessions,
    addPracticeSession,
    addPiecePracticed,
    getRootURL,
};
