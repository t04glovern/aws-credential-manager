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

  // Fetch AWS profiles when the component mounts
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
    // Reset the form fields when the selected profile changes
    setProfileName(selectedProfile);
    setAccessKeyId("");
    setSecretAccessKey("");
    setSessionToken("");
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
      alert('Profile updated successfully!');
    } catch (error) {
      console.error("Failed to add or edit AWS profile:", error);
      alert('Failed to update profile.');
    }
  };

  const handleDeleteProfile = async () => {
    if (selectedProfile) {
      try {
        await invoke("delete_aws_profile", { profile: selectedProfile });
        alert("Profile deleted successfully");
        // Refresh the profiles list
        const profiles = await invoke("list_aws_profiles");
        setAwsProfiles(profiles as string[]);
        setSelectedProfile("");
      } catch (error) {
        console.error("Failed to delete AWS profile:", error);
        alert("Failed to delete profile");
      }
    }
  };

  return (
    <div className="container">
      <h1>Welcome to AWS Credential Checker!</h1>

      {/* AWS Profile Selection and Identity Check */}
      <div>
        <label htmlFor="profile-select">Select AWS Profile:</label>
        <select
          id="profile-select"
          value={selectedProfile}
          onChange={(e) => setSelectedProfile(e.target.value)}
          disabled={awsProfiles.length === 0}
        >
          <option value="">--Please choose an AWS profile--</option>
          {awsProfiles.map((profile) => (
            <option key={profile} value={profile}>{profile}</option>
          ))}
        </select>
        <button onClick={checkIdentity} disabled={!selectedProfile}>Check Identity</button>
        <button onClick={handleDeleteProfile} disabled={!selectedProfile}>Delete Profile</button>
      </div>

      {/* AWS Identity Information Display */}
      <div>
        <h2>Identity Information:</h2>
        <textarea
          value={identityInfo}
          readOnly
          rows={10}
          cols={50}
          style={{ resize: "none" }}
        ></textarea>
      </div>

      {/* Form for Adding/Editing AWS Profiles */}
      <div>
        <h2>{selectedProfile ? "Edit" : "Add"} AWS Profile:</h2>
        {!selectedProfile && (
          <input
            type="text"
            placeholder="Profile Name"
            value={profileName}
            onChange={(e) => setProfileName(e.target.value)}
          />
        )}
        <input
          type="text"
          placeholder="Access Key ID"
          value={accessKeyId}
          onChange={(e) => setAccessKeyId(e.target.value)}
        />
        <input
          type="text"
          placeholder="Secret Access Key"
          value={secretAccessKey}
          onChange={(e) => setSecretAccessKey(e.target.value)}
        />
        <input
          type="text"
          placeholder="Session Token (Optional)"
          value={sessionToken}
          onChange={(e) => setSessionToken(e.target.value)}
        />
        <button onClick={handleAddOrEditProfile} disabled={!profileName || !accessKeyId || !secretAccessKey}>
          {selectedProfile ? "Update" : "Add"} Profile
        </button>
      </div>
    </div>
  );
}

export default App;
