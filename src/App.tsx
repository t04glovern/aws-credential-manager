import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  const [awsProfiles, setAwsProfiles] = useState<string[]>([]);
  const [selectedProfile, setSelectedProfile] = useState<string>("");
  const [profileName, setProfileName] = useState<string>("");
  const [identityInfo, setIdentityInfo] = useState<string>("");
  const [accessKeyId, setAccessKeyId] = useState<string>("");
  const [secretAccessKey, setSecretAccessKey] = useState<string>("");
  const [sessionToken, setSessionToken] = useState<string>("");

  useEffect(() => {
    const fetchProfiles = async () => {
      try {
        const profiles = await invoke("list_aws_profiles");
        setAwsProfiles(profiles as string[]);
      } catch (error) {
        console.error("Failed to fetch AWS profiles:", error);
      }
    };

    fetchProfiles();
  }, []);

  useEffect(() => {
    const fetchProfileDetails = async () => {
      if (!selectedProfile) {
        setProfileName("");
        setAccessKeyId("");
        setSecretAccessKey("");
        setSessionToken("");
      } else {
        try {
          const details = await invoke("get_aws_profile_details", { profile: selectedProfile });
          const { access_key_id, secret_access_key, session_token } = details as any;

          setProfileName(selectedProfile);
          setAccessKeyId(access_key_id);
          setSecretAccessKey(secret_access_key);
          setSessionToken(session_token || "");
        } catch (error) {
          console.error("Failed to fetch profile details:", error);
        }
      }
    };

    fetchProfileDetails();
  }, [selectedProfile]);

  const checkIdentity = async () => {
    try {
      const identity = await invoke("check_aws_identity", { profile: selectedProfile });
      setIdentityInfo(identity as string);
    } catch (error) {
      console.error("Failed to check AWS identity:", error);
    }
  };

  const handleAddOrEditProfile = async () => {
    try {
      const profileData = {
        profileName: profileName,
        accessKeyId: accessKeyId,
        secretAccessKey: secretAccessKey,
        sessionToken: sessionToken || undefined,
      };

      await invoke("add_or_edit_aws_profile", { profile: profileData });

      // Refresh the profiles list to include the newly added/updated profile
      const profiles = await invoke("list_aws_profiles");
      setAwsProfiles(profiles as string[]);
      setSelectedProfile(profileName); // Select the newly added/updated profile
    } catch (error) {
      console.error("Failed to add or edit AWS profile:", error);
    }
  };

  const handleDeleteProfile = async () => {
    if (selectedProfile) {
      try {
        await invoke("delete_aws_profile", { profile: selectedProfile });
        // Refresh the profiles list
        const profiles = await invoke("list_aws_profiles");
        setAwsProfiles(profiles as string[]);
        setSelectedProfile("");
      } catch (error) {
        console.error("Failed to delete AWS profile:", error);
      }
    }
  };

  return (
    <div className="container mx-auto px-4">
      <h1 className="text-3xl font-bold my-4 text-primary">AWS Credential Manager</h1>

      {/* AWS Profile Selection and Identity Check */}
      <div className="flex flex-col md:flex-row justify-between items-center mb-4 space-y-2 md:space-y-0 md:space-x-2">
        <select
          id="profile-select"
          className="form-select mt-1 block w-full border-neutral rounded-md shadow-sm h-12"
          value={selectedProfile}
          onChange={(e) => setSelectedProfile(e.target.value)}
          disabled={awsProfiles.length === 0}
        >
          <option value="">-- Select AWS profile --</option>
          {awsProfiles.map((profile) => (
            <option key={profile} value={profile}>{profile}</option>
          ))}
        </select>
        <button className="bg-primary hover:bg-secondary text-white font-bold py-2 px-4 rounded w-full md:w-1/4 h-12" onClick={checkIdentity} disabled={!selectedProfile}>Check</button>
        <button className="bg-success hover:bg-warning text-white font-bold py-2 px-4 rounded w-full md:w-1/4 h-12" onClick={handleDeleteProfile} disabled={!selectedProfile}>Delete</button>
      </div>

      {/* AWS Identity Information Display */}
      <div className="mb-8">
        <h2 className="text-2xl font-semibold mb-4 text-primary">Identity Information:</h2>
        <div className="w-full p-3 border-neutral rounded-md shadow-sm resize-none h-36 bg-gray-100 text-gray-600">
          {identityInfo || <span className="italic">No identity information available.</span>}
        </div>
      </div>

      {/* Form for Adding/Editing AWS Profiles */}
      <div>
        <h2 className="text-2xl font-semibold mb-4 text-primary">{selectedProfile ? "Edit" : "Add"} AWS Profile:</h2>
        <div className="flex flex-col space-y-4">
          {!selectedProfile && (
            <input
              type="text"
              placeholder="Profile Name"
              className="form-input mt-1 block w-full pl-3 py-2 border-neutral rounded-md shadow-sm h-12"
              value={profileName}
              onChange={(e) => setProfileName(e.target.value)}
            />
          )}
          <input
            type="text"
            placeholder="Access Key ID"
            className="form-input mt-1 block w-full pl-3 py-2 border-neutral rounded-md shadow-sm h-12"
            value={accessKeyId}
            onChange={(e) => setAccessKeyId(e.target.value)}
          />
          <input
            type="password"
            placeholder="Secret Access Key"
            className="form-input mt-1 block w-full pl-3 py-2 border-neutral rounded-md shadow-sm h-12"
            value={secretAccessKey}
            onChange={(e) => setSecretAccessKey(e.target.value)}
          />
          <input
            type="password"
            placeholder="Session Token (Optional)"
            className="form-input mt-1 block w-full pl-3 py-2 border-neutral rounded-md shadow-sm h-12"
            value={sessionToken}
            onChange={(e) => setSessionToken(e.target.value)}
          />
          <button
            onClick={handleAddOrEditProfile}
            disabled={!profileName || !accessKeyId || !secretAccessKey}
            className="bg-primary hover:bg-secondary text-white font-bold py-2 px-4 rounded w-full h-12"
          >
            {selectedProfile ? "Update" : "Add"} Profile
          </button>
        </div>
      </div>
    </div>
  );
}

export default App;