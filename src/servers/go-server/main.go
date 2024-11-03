package main

import (
	"encoding/json"
	"log"
	"net/http"
	"strconv"

	"github.com/gorilla/mux"
	"github.com/rs/cors"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

// User represents a user in the system
type User struct {
	ID          uint   `json:"id" gorm:"primaryKey"`
	Name        string `json:"name"`
	HoursWorked int    `json:"hoursWorked"`
}

// UpdateUserRequest represents the fields that can be updated
type UpdateUserRequest struct {
	Name *string `json:"name"`
}

var db *gorm.DB
var err error

func main() {
	// Initialize the database
	db, err = gorm.Open(sqlite.Open("test.db"), &gorm.Config{})
	if err != nil {
		log.Fatal(err)
	}
	db.AutoMigrate(&User{})

	// Initialize the router
	router := mux.NewRouter()

	// Define the routes
	router.HandleFunc("/users", getUsers).Methods("GET")
	router.HandleFunc("/users/{id}", getUserByID).Methods("GET")
	router.HandleFunc("/users", addUser).Methods("POST")
	router.HandleFunc("/users/{id}", updateUserByID).Methods("PUT")
	router.HandleFunc("/users/{id}", updateUserHours).Methods("PATCH")
	router.HandleFunc("/users", deleteAllUsers).Methods("DELETE")
	router.HandleFunc("/users/{id}", deleteUserByID).Methods("DELETE")

	// Set up CORS
	c := cors.New(cors.Options{
		AllowedOrigins:   []string{"*"},
		AllowedMethods:   []string{"GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"},
		AllowedHeaders:   []string{"Content-Type", "Authorization"},
		AllowCredentials: true,
	})

	// Start the server
	handler := c.Handler(router)
	log.Println("Server is running on port 5004")
	log.Fatal(http.ListenAndServe(":5004", handler))
}

func getUsers(w http.ResponseWriter, r *http.Request) {
	var users []User
	db.Find(&users)
	json.NewEncoder(w).Encode(users)
}

func getUserByID(w http.ResponseWriter, r *http.Request) {
	params := mux.Vars(r)
	id, _ := strconv.Atoi(params["id"])
	var user User
	if result := db.First(&user, id); result.Error != nil {
		http.Error(w, "User not found", http.StatusNotFound)
		return
	}
	json.NewEncoder(w).Encode(user)
}

func addUser(w http.ResponseWriter, r *http.Request) {
	var user User
	json.NewDecoder(r.Body).Decode(&user)
	if user.Name == "" {
		http.Error(w, "Name is required and must be a non-empty string", http.StatusBadRequest)
		return
	}
	db.Create(&user)
	json.NewEncoder(w).Encode(user)
}

func updateUserByID(w http.ResponseWriter, r *http.Request) {
	params := mux.Vars(r)
	id, _ := strconv.Atoi(params["id"])
	var user User
	if result := db.First(&user, id); result.Error != nil {
		http.Error(w, "User not found", http.StatusNotFound)
		return
	}

	var updateUserRequest struct {
		Name *string `json:"name"`
	}
	json.NewDecoder(r.Body).Decode(&updateUserRequest)

	if updateUserRequest.Name != nil && *updateUserRequest.Name != "" {
		user.Name = *updateUserRequest.Name
		log.Printf("Updated user name to: %s", user.Name)
	}

	db.Save(&user)
	json.NewEncoder(w).Encode(user)
}

func updateUserHours(w http.ResponseWriter, r *http.Request) {
	params := mux.Vars(r)
	id, _ := strconv.Atoi(params["id"])
	var user User
	if result := db.First(&user, id); result.Error != nil {
		http.Error(w, "User not found", http.StatusNotFound)
		return
	}

	var updateHoursRequest struct {
		HoursToAdd int `json:"hoursToAdd"`
	}
	json.NewDecoder(r.Body).Decode(&updateHoursRequest)

	if updateHoursRequest.HoursToAdd > 0 {
		user.HoursWorked += updateHoursRequest.HoursToAdd
		log.Printf("Added %d hours. Total hours worked: %d", updateHoursRequest.HoursToAdd, user.HoursWorked)
		db.Save(&user)
		json.NewEncoder(w).Encode(user)
	} else {
		http.Error(w, "Invalid hoursToAdd value", http.StatusBadRequest)
	}
}

func deleteAllUsers(w http.ResponseWriter, r *http.Request) {
	db.Exec("DELETE FROM users")
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode([]User{})
}

func deleteUserByID(w http.ResponseWriter, r *http.Request) {
	params := mux.Vars(r)
	id, _ := strconv.Atoi(params["id"])
	var user User
	if result := db.First(&user, id); result.Error != nil {
		http.Error(w, "User not found", http.StatusNotFound)
		return
	}
	db.Delete(&user)
	w.WriteHeader(http.StatusNoContent)
}
